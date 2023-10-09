use std::result::Result;

use common::BridgeSide;
use common_eth::EthPrivateKey;
use common_sentinel::{
    BroadcastChannelMessages,
    BroadcasterBroadcastChannelMessages,
    BroadcasterMessages,
    ConfigT,
    Env,
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    UserOp,
    UserOpCancellationSignature,
    UserOpError,
    UserOps,
    WebSocketMessages,
    WebSocketMessagesCancelUserOpArgs,
    WebSocketMessagesEncodable,
    WebSocketMessagesGetCancellableUserOpArgs,
};
use ethereum_types::H256 as EthHash;
use tokio::{
    sync::{
        broadcast::{Receiver as MpMcRx, Sender as MpMcTx},
        mpsc::{Receiver as MpscRx, Sender as MpscTx},
    },
    time::{sleep, Duration},
};

#[allow(clippy::too_many_arguments)]
async fn cancel_user_op(
    op: UserOp,
    nonce: u64,
    //balance: U256, // FIXME Make this optional. Race the getter. If it's none skip the balance check
    gas_price: u64,
    gas_limit: usize,
    config: &SentinelConfig,
    broadcaster_pk: &EthPrivateKey,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    websocket_tx: MpscTx<WebSocketMessages>,
) -> Result<EthHash, SentinelError> {
    // FIXME re-instate the balance checks
    // NOTE: First we check we can afford the tx
    //op.check_affordability(balance, gas_limit, gas_price)?;

    let side = op.destination_side();
    let pnetwork_hub = config.pnetwork_hub(&side);
    debug!("cancelling user op on side: {side} nonce: {nonce} gas price: {gas_price}");

    /*
    let (msg, rx) = EthRpcMessages::get_user_op_state_msg(side, op.clone(), pnetwork_hub);
    eth_rpc_tx.send(msg).await?;
    let user_op_smart_contract_state = rx.await??;
    debug!("user op state before cancellation: {user_op_smart_contract_state}");

    if !user_op_smart_contract_state.is_cancellable() {
        return Err(UserOpError::CannotCancel(Box::new(op)).into());
    }
    */

    let mcids = vec![config.native().mcid(), config.host().mcid()];
    let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetUserOpCancellationSiganture(Box::new(
        WebSocketMessagesCancelUserOpArgs::new(mcids.clone(), op.clone()),
    )));
    websocket_tx.send(msg).await?;

    warn!("here are the mcids: {mcids:?}"); // FIXME rm

    let cancellation_sig = UserOpCancellationSignature::try_from(tokio::select! {
        response = rx => response?,
        _ = sleep(Duration::from_secs(*config.core().timeout())) => {
            let m = "getting cancellation signature";
            error!("timed out whilst {m}");
            Err(SentinelError::Timedout(m.into()))
        }
    }?)?;

    warn!("cancellation signature: {mcids:?}"); // FIXME rm

    let signed_tx = op.get_cancellation_tx(
        nonce,
        gas_price,
        gas_limit,
        &pnetwork_hub,
        &config.chain_id(&side),
        broadcaster_pk,
        &cancellation_sig,
    )?;

    debug!("signed tx: {}", signed_tx.serialize_hex());

    let (msg, rx) = EthRpcMessages::get_push_tx_msg(signed_tx, side);
    eth_rpc_tx.send(msg).await?;
    let tx_hash = rx.await??;

    info!("tx hash: {tx_hash}");
    Ok(tx_hash)
}

async fn get_gas_price(
    config: &SentinelConfig,
    side: BridgeSide,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
) -> Result<u64, SentinelError> {
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

async fn cancel_user_ops(
    config: &SentinelConfig,
    websocket_tx: MpscTx<WebSocketMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    native_broadcaster_pk: &EthPrivateKey,
    host_broadcaster_pk: &EthPrivateKey,
) -> Result<(), SentinelError> {
    info!("handling user op cancellation request...");

    let max_delta = config.core().max_cancellable_time_delta();
    let args = WebSocketMessagesGetCancellableUserOpArgs::new(*max_delta, vec![
        // NOTE/FIXME For now, the ordering of these is very important since they're _assumed_ to
        // be in the order of native/host. Eventually we will be able to deal with > 2 chains, at
        // which point the ordering will stop mattering.
        config.native().metadata_chain_id(),
        config.host().metadata_chain_id(),
    ]);
    let encodable_msg = WebSocketMessagesEncodable::GetCancellableUserOps(Box::new(args));
    let (msg, rx) = WebSocketMessages::new(encodable_msg);
    websocket_tx.send(msg).await?;

    let cancellable_user_ops = UserOps::try_from(tokio::select! {
        response = rx => response?,
        _ = sleep(Duration::from_secs(*config.core().timeout())) => {
            let m = "getting cancellable user ops";
            error!("timed out whilst {m}");
            Err(SentinelError::Timedout(m.into()))
        }
    }?)?;

    if cancellable_user_ops.is_empty() {
        warn!("no user ops to cancel");
        return Ok(());
    }

    let host_address = host_broadcaster_pk.to_address();
    let native_address = native_broadcaster_pk.to_address();

    let (host_msg, host_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Host, host_address);
    eth_rpc_tx.send(host_msg).await?;
    let mut host_nonce = host_rx.await??;

    let (native_msg, native_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Native, native_address);
    eth_rpc_tx.send(native_msg).await?;
    let mut native_nonce = native_rx.await??;

    let host_gas_price = get_gas_price(config, BridgeSide::Host, eth_rpc_tx.clone()).await?;
    let native_gas_price = get_gas_price(config, BridgeSide::Native, eth_rpc_tx.clone()).await?;

    let host_gas_limit = config.gas_limit(&BridgeSide::Host);
    let native_gas_limit = config.gas_limit(&BridgeSide::Native);

    /*
    let (host_balance_msg, host_balance_rx) = EthRpcMessages::get_eth_balance_msg(BridgeSide::Host, host_address);
    let (native_balance_msg, native_balance_rx) =
        EthRpcMessages::get_eth_balance_msg(BridgeSide::Native, native_address);
    eth_rpc_tx.send(native_balance_msg).await?;
    let mut host_balance = host_balance_rx.await??;
    eth_rpc_tx.send(host_balance_msg).await?;
    let mut native_balance = native_balance_rx.await??;
    */

    let err_msg = "error cancelling user op ";

    for op in cancellable_user_ops.iter() {
        match op.destination_side() {
            BridgeSide::Native => {
                let uid = op.uid()?;
                match cancel_user_op(
                    op.clone(),
                    native_nonce,
                    //native_balance,
                    native_gas_price,
                    native_gas_limit,
                    config,
                    native_broadcaster_pk,
                    eth_rpc_tx.clone(),
                    websocket_tx.clone(),
                )
                .await
                {
                    Err(e) => {
                        error!("{err_msg} {uid} {e}");
                    },
                    Ok(tx_hash) => {
                        info!(
                            "user op {uid} cancelled successfully @ tx {}",
                            hex::encode(tx_hash.as_bytes())
                        );
                    },
                }
                native_nonce += 1;
                //native_balance -= UserOp::get_tx_cost(native_gas_limit, native_gas_price);
            },
            BridgeSide::Host => {
                let uid = op.uid()?;
                match cancel_user_op(
                    op.clone(),
                    host_nonce,
                    //host_balance,
                    host_gas_price,
                    host_gas_limit,
                    config,
                    host_broadcaster_pk,
                    eth_rpc_tx.clone(),
                    websocket_tx.clone(),
                )
                .await
                {
                    Err(e) => {
                        error!("{err_msg} {uid} {e}");
                    },
                    Ok(tx_hash) => {
                        info!(
                            "user op {uid} cancelled successfully @ tx {}",
                            hex::encode(tx_hash.as_bytes())
                        );
                    },
                }
                host_nonce += 1;
                //host_balance -= UserOp::get_tx_cost(host_gas_limit, host_gas_price);
            },
        }
    }

    Ok(())
}

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: MpMcRx<BroadcastChannelMessages>,
) -> Result<BroadcasterBroadcastChannelMessages, SentinelError> {
    // NOTE: This loops continuously listening to the broadcasting channel, and only returns if we
    // receive a pertinent message. This way, other messages won't cause early returns in the main
    // tokios::select, so then the main_loop can continue doing its work.
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::Broadcaster(msg)) => break 'broadcast_channel_loop Ok(msg),
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for us
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

const CANCELLABLE_OPS_CHECK_FREQUENCY: u64 = 120; // FIXME make configurable! Make updatable whilst running too!

async fn cancellation_loop(frequency: u64, broadcaster_tx: MpscTx<BroadcasterMessages>) -> Result<(), SentinelError> {
    // NOTE: This loop runs to send messages to the broadcaster at a configruable frequency to tell
    // it to try and cancel any cancellable user ops. It should never return, except in error.
    'cancellation_loop: loop {
        sleep(Duration::from_secs(frequency)).await;
        warn!("{frequency}s has elapsed - sending message to cancel any cancellable user ops");
        match broadcaster_tx.send(BroadcasterMessages::CancelUserOps).await {
            Ok(_) => continue 'cancellation_loop,
            Err(e) => break 'cancellation_loop Err(e.into()),
        }
    }
}

pub async fn broadcaster_loop(
    mut rx: MpscRx<BroadcasterMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    config: SentinelConfig,
    broadcast_channel_tx: MpMcTx<BroadcastChannelMessages>,
    websocket_tx: MpscTx<WebSocketMessages>,
    broadcaster_tx: MpscTx<BroadcasterMessages>,
) -> Result<(), SentinelError> {
    let name = "broadcaster";

    let mut core_is_connected = false;
    let mut broadcaster_is_enabled = false;

    warn!("{name} not active yet due to no core connection");

    Env::init()?;
    let host_broadcaster_pk = Env::get_host_broadcaster_private_key()?;
    let native_broadcaster_pk = Env::get_native_broadcaster_private_key()?;

    'broadcaster_loop: loop {
        tokio::select! {
            r = cancellation_loop(CANCELLABLE_OPS_CHECK_FREQUENCY, broadcaster_tx.clone()), if (broadcaster_is_enabled && core_is_connected) => {
                let sleep_time = 30;
                match r {
                    Ok(_) => {
                        warn!("broadcaster cancellation loop returned Ok(()) for some reason");
                    },
                    Err(e) => {
                        error!("broadcaster cancellation loop error: {e}");
                    }
                }
                warn!("sleeping for {sleep_time}s and restarting broadcaster loop");
                sleep(Duration::from_secs(sleep_time)).await;
                continue 'broadcaster_loop
            },
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe()) => {
                match r {
                    Ok(msg) => {
                        let note = format!("(core is currently {}connected)", if core_is_connected { "" } else { "not "});
                        match msg {
                            BroadcasterBroadcastChannelMessages::Stop => {
                                warn!("msg received to stop the {name} {note}");
                                broadcaster_is_enabled = false;
                                continue 'broadcaster_loop
                            },
                            BroadcasterBroadcastChannelMessages::Start => {
                                warn!("msg received to start the {name} {note}");
                                broadcaster_is_enabled = true;
                                continue 'broadcaster_loop
                            },
                            BroadcasterBroadcastChannelMessages::CoreConnected => {
                                warn!("core connected message received in {name} {note}");
                                core_is_connected = true;
                                continue 'broadcaster_loop
                            },
                            BroadcasterBroadcastChannelMessages::CoreDisconnected => {
                                warn!("core disconnected message received in {name} {note}");
                                core_is_connected = false;
                                continue 'broadcaster_loop
                            },
                        }
                    },
                    Err(e) => break 'broadcaster_loop Err(e),
                }
            },
            r = rx.recv() , if broadcaster_is_enabled && core_is_connected => match r {
                Some(BroadcasterMessages::CancelUserOps) => {
                    match cancel_user_ops(
                        &config,
                        websocket_tx.clone(),
                        eth_rpc_tx.clone(),
                        &native_broadcaster_pk,
                        &host_broadcaster_pk,
                    ).await {
                        Ok(_) => {
                            info!("finished handling user op cancellation request");
                        }
                        Err(SentinelError::UserOp(boxed_user_op_error)) => match *boxed_user_op_error {
                            UserOpError::InsufficientBalance { have, need } => {
                                error!("!!! WARNING !!!");
                                error!("!!! WARNING !!!");
                                error!("!!! WARNING !!!");
                                warn!(">>> insufficient balance to cancel a user op - have: {have}, need: {need} <<<");
                                error!("!!! WARNING !!!");
                                error!("!!! WARNING !!!");
                                error!("!!! WARNING !!!");
                                continue 'broadcaster_loop
                            },
                            e => {
                                error!("unhandled user op error: {e}");
                                break 'broadcaster_loop Err(e.into())
                            }
                        },
                        Err(e) => {
                            error!("{e}");
                        }
                    };
                    continue 'broadcaster_loop
                },
                None => {
                    let m = "all {name} senders dropped!";
                    warn!("{m}");
                    break 'broadcaster_loop Err(SentinelError::Custom(name.into()))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'broadcaster_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if broadcaster_is_enabled { "en" } else { "dis" });
                continue 'broadcaster_loop
            },
        }
    }
}
