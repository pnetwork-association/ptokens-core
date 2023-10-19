use common::BridgeSide;
use common_eth::EthPrivateKey;
use common_sentinel::{
    call_core,
    BroadcastChannelMessages,
    ChallengeAndResponseInfo,
    ChallengeAndResponseInfos,
    ChallengeResponderBroadcastChannelMessages,
    ChallengeResponderMessages,
    ConfigT,
    Env,
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
};
use tokio::time::{sleep, Duration};

use crate::type_aliases::{
    BroadcastChannelRx,
    BroadcastChannelTx,
    ChallengeResponderRx,
    ChallengeResponderTx,
    CoreCxnStatus,
    EthRpcTx,
    WebSocketTx,
};

async fn respond_to_challenge(
    info: &ChallengeAndResponseInfo,
    nonce: u64,
    //balance: U256, // FIXME Make this optional. Race the getter. If it's none skip the balance check
    gas_price: u64,
    gas_limit: usize,
    config: &SentinelConfig,
    broadcaster_pk: &EthPrivateKey,
    eth_rpc_tx: EthRpcTx,
    websocket_tx: WebSocketTx,
) -> Result<(), SentinelError> {
    let id = info.challenge().id()?;
    let mcid = *info.challenge().mcid();
    let side = if config.native().mcid() == mcid {
        BridgeSide::Native
    } else {
        BridgeSide::Host
    };
    let hub = config.pnetwork_hub(&side);
    let signed_tx = info.challenge().to_solve_challenge_tx(
        nonce,
        gas_price,
        gas_limit,
        &mcid,
        &hub,
        broadcaster_pk,
        info.response_info(),
    )?;
    // NOTE: We're still stuck with the host/native paradigm for the time being.

    let (msg, rx) = EthRpcMessages::get_push_tx_msg(signed_tx, side);
    eth_rpc_tx.send(msg).await?;
    let tx_hash = rx.await??;

    info!("tx hash: {tx_hash}");

    call_core(
        *config.core().timeout(),
        websocket_tx.clone(),
        WebSocketMessagesEncodable::SetChallengesToSolved(vec![id]),
    )
    .await?;

    Ok(())
}

async fn get_gas_price(config: &SentinelConfig, side: BridgeSide, eth_rpc_tx: EthRpcTx) -> Result<u64, SentinelError> {
    let p = if let Some(p) = config.gas_price(&side) {
        debug!("using {side} gas price from config: {p}");
        p
    } else {
        let (msg, rx) = EthRpcMessages::get_gas_price_msg(side);
        eth_rpc_tx.send(msg).await?;
        let p = rx.await??;
        debug!("using {side} gas price from rpc: {p}");
        p
    };
    Ok(p)
}

async fn respond_to_challenges(
    config: &SentinelConfig,
    websocket_tx: WebSocketTx,
    core_timeout: &u64,
    native_eth_rpc_tx: EthRpcTx,
    host_eth_rpc_tx: EthRpcTx,
    pk: &EthPrivateKey,
) -> Result<(), SentinelError> {
    info!("responding to challenges...");
    let unsolved_challenges = ChallengeAndResponseInfos::try_from(
        call_core(
            *core_timeout,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetUnsolvedChallenges,
        )
        .await?,
    )?;

    if unsolved_challenges.is_empty() {
        warn!("no challenges to respond to");
        return Ok(());
    }

    let address = pk.to_address();
    let (native_msg, native_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Native, address);
    let (host_msg, host_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Host, address);
    native_eth_rpc_tx.send(native_msg).await?;
    host_eth_rpc_tx.send(host_msg).await?;
    let mut native_nonce = native_rx.await??;
    let mut host_nonce = host_rx.await??;

    let native_gas_price = get_gas_price(config, BridgeSide::Native, native_eth_rpc_tx.clone()).await?;
    let host_gas_price = get_gas_price(config, BridgeSide::Host, host_eth_rpc_tx.clone()).await?;

    let gas_limit = 1_000_000; // FIXME make configurable for this

    for challenge_info in unsolved_challenges.iter() {
        let side = if config.native().mcid() == *challenge_info.challenge().mcid() {
            BridgeSide::Native
        } else {
            BridgeSide::Host
        };
        respond_to_challenge(
            challenge_info,
            if side.is_native() { native_nonce } else { host_nonce },
            if side.is_native() {
                native_gas_price
            } else {
                host_gas_price
            },
            gas_limit,
            config,
            pk,
            if side.is_native() {
                native_eth_rpc_tx.clone()
            } else {
                host_eth_rpc_tx.clone()
            },
            websocket_tx.clone(),
        )
        .await?;

        if side.is_native() {
            native_nonce += 1
        } else {
            host_nonce += 1
        };
    }

    Ok(())
}

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: BroadcastChannelRx,
) -> Result<ChallengeResponderBroadcastChannelMessages, SentinelError> {
    // NOTE: This loops continuously listening to the broadcasting channel, and only returns if we
    // receive a pertinent message. This way, other messages won't cause early returns in the main
    // tokios::select, so that the main_loop can continue doing its work.
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::ChallengeResponder(msg)) => break 'broadcast_channel_loop Ok(msg),
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for us
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

async fn respond_to_challenges_loop(
    frequency: &u64,
    challenge_responder_tx: ChallengeResponderTx,
    core_cxn_status: &CoreCxnStatus,
    challenge_responder_is_enabled: &bool,
) -> Result<(), SentinelError> {
    // NOTE: This loop runs to send messages to the responder loop at a configurable frequency to tell
    // it to respond to any open challenges. It should never return, except in error.
    'respond_to_challenge_loop: loop {
        info!("challenge responder loop sleeping for {frequency}s...");
        sleep(Duration::from_secs(*frequency)).await;

        if !core_cxn_status {
            warn!("core is currently not connected so cannot respond to challenges");
            continue 'respond_to_challenge_loop;
        }

        if !challenge_responder_is_enabled {
            warn!("challenge responder currently disabled so will not respond to challenges");
            continue 'respond_to_challenge_loop;
        }

        info!("{frequency}s has elapsed - sending message to publish status...");
        match challenge_responder_tx
            .send(ChallengeResponderMessages::RespondToChallenges)
            .await
        {
            Ok(_) => continue 'respond_to_challenge_loop,
            Err(e) => break 'respond_to_challenge_loop Err(e.into()),
        }
    }
}

pub async fn challenge_responder_loop(
    config: SentinelConfig,
    mut challenge_responder_rx: ChallengeResponderRx,
    challenge_responder_tx: ChallengeResponderTx,
    broadcast_channel_tx: BroadcastChannelTx,
    websocket_tx: WebSocketTx,
    native_eth_rpc_tx: EthRpcTx,
    host_eth_rpc_tx: EthRpcTx,
) -> Result<(), SentinelError> {
    let name = "challenge responder loop";

    let mut core_is_connected = false;
    let mut challenge_responder_is_enabled = true;
    let core_timeout = *config.core().timeout(); // TODO Make updateable via rpc call
    let mut frequency = *config.core().challenge_response_frequency();

    Env::init()?;
    // NOTE: We don't use sides in the broadcasting pk management
    let pk = Env::get_native_broadcaster_private_key()?;

    'challenge_response_loop: loop {
        tokio::select! {
            r = respond_to_challenges_loop(
                    &frequency,
                    challenge_responder_tx.clone(),
                    &core_is_connected,
                    &challenge_responder_is_enabled,
                ) => {
                match r {
                    Ok(_) => { warn!("publish status loop returned Ok(()) for some reason") },
                    Err(e) => { error!("publish status publisher loop error: {e}") },
                };

                let sleep_time = 30; // FIXME make configurable
                warn!("sleeping for {sleep_time}s and restarting status loop");
                sleep(Duration::from_secs(sleep_time)).await;
                continue 'challenge_response_loop
            },
            r = challenge_responder_rx.recv() => match r {
                Some(ChallengeResponderMessages::RespondToChallenges) => {
                    if !core_is_connected {
                        warn!("not responding to open challenges because no core is connected");
                        continue 'challenge_response_loop
                    } else {
                        match respond_to_challenges(
                            &config,
                            websocket_tx.clone(),
                            &core_timeout,
                            native_eth_rpc_tx.clone(),
                            host_eth_rpc_tx.clone(),
                            &pk,
                        ).await {
                            Ok(_) => continue 'challenge_response_loop,
                            Err(e) => break 'challenge_response_loop Err(e)
                        }
                    }
                },
                Some(ChallengeResponderMessages::SetChallengeResponseFrequency(new_frequency)) => {
                    frequency = new_frequency;
                    info!("updated challenge responding frequency to {new_frequency}");
                    continue 'challenge_response_loop
                },
                None => {
                    let m = "all {name} senders dropped!";
                    warn!("{m}");
                    break 'challenge_response_loop Err(SentinelError::Custom(name.into()))
                },
            },
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe()) => {
                match r {
                    Err(e) => break 'challenge_response_loop Err(e),
                    Ok(msg) => {
                        let note = format!("(core is currently {}connected)", if core_is_connected { "" } else { "not "});
                        match msg {
                            ChallengeResponderBroadcastChannelMessages::Stop => {
                                warn!("msg received to stop the challenge responder {note}");
                                challenge_responder_is_enabled = false;
                                continue 'challenge_response_loop
                            },
                            ChallengeResponderBroadcastChannelMessages::Start => {
                                warn!("msg received to start the challenge responder {note}");
                                challenge_responder_is_enabled = true;
                                continue 'challenge_response_loop
                            },
                            ChallengeResponderBroadcastChannelMessages::CoreConnected => {
                                warn!("core connected message received in {name} {note}");
                                core_is_connected = true;
                                continue 'challenge_response_loop
                            },
                            ChallengeResponderBroadcastChannelMessages::CoreDisconnected => {
                                warn!("core disconnected message received in {name} {note}");
                                core_is_connected = false;
                                continue 'challenge_response_loop
                            },
                        }
                    },
                }
            },

            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'challenge_response_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if challenge_responder_is_enabled { "en" } else { "dis" });
                continue 'challenge_response_loop
            },
        }
    }
}
