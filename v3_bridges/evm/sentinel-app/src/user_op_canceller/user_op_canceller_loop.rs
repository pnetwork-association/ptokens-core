use std::result::Result;

use common_eth::EthPrivateKey;
use common_sentinel::{
    call_core,
    BroadcastChannelMessages,
    Env,
    EthRpcMessages,
    NetworkId,
    SentinelConfig,
    SentinelError,
    UserOp,
    UserOpCancellationSignature,
    UserOpCancellerBroadcastChannelMessages,
    UserOpCancellerMessages,
    UserOpError,
    UserOps,
    WebSocketMessagesCancelUserOpArgs,
    WebSocketMessagesEncodable,
    WebSocketMessagesGetCancellableUserOpArgs,
};
use ethereum_types::H256 as EthHash;
use tokio::time::{sleep, Duration};

use crate::type_aliases::{
    BroadcastChannelRx,
    BroadcastChannelTx,
    EthRpcTx,
    UserOpCancellerRx,
    UserOpCancellerTx,
    WebSocketTx,
};

async fn cancel_user_op(
    op: UserOp,
    nonce: u64,
    //balance: U256, // FIXME Make this optional. Race the getter. If it's none skip the balance check
    gas_price: u64,
    gas_limit: usize,
    config: &SentinelConfig,
    broadcaster_pk: &EthPrivateKey,
    eth_rpc_tx: EthRpcTx,
    websocket_tx: WebSocketTx,
) -> Result<EthHash, SentinelError> {
    // FIXME re-instate the balance checks
    // NOTE: First we check we can afford the tx
    //op.check_affordability(balance, gas_limit, gas_price)?;

    let destination_network_id = op.destination_network_id();
    let pnetwork_hub = config.pnetwork_hub_from_network_id(&destination_network_id)?;
    debug!("cancelling user op on network: {destination_network_id} nonce: {nonce} gas price: {gas_price}");

    let (msg, rx) = EthRpcMessages::get_user_op_state_msg(destination_network_id, op.clone(), pnetwork_hub);
    eth_rpc_tx.send(msg).await?;
    let user_op_smart_contract_state = rx.await??;
    debug!("user op state before cancellation: {user_op_smart_contract_state}");

    if !user_op_smart_contract_state.is_cancellable() {
        return Err(UserOpError::CannotCancel(Box::new(op)).into());
    }

    let network_ids = config.network_ids()?;
    let msg = WebSocketMessagesEncodable::GetUserOpCancellationSignature(Box::new(
        WebSocketMessagesCancelUserOpArgs::new(network_ids, op.clone()),
    ));

    let cancellation_sig =
        UserOpCancellationSignature::try_from(call_core(*config.core().timeout(), websocket_tx.clone(), msg).await?)?;

    let signed_tx = op.get_cancellation_tx(
        nonce,
        gas_price,
        gas_limit,
        &pnetwork_hub,
        &config.eth_chain_id_from_network_id(&destination_network_id)?,
        broadcaster_pk,
        &cancellation_sig,
    )?;

    debug!("signed tx: {}", signed_tx.serialize_hex());

    let (msg, rx) = EthRpcMessages::get_push_tx_msg(signed_tx, destination_network_id);
    eth_rpc_tx.send(msg).await?;
    let tx_hash = rx.await??;

    info!("tx hash: {tx_hash}");
    Ok(tx_hash)
}

async fn get_gas_price(
    config: &SentinelConfig,
    network_id: &NetworkId,
    eth_rpc_tx: EthRpcTx,
) -> Result<u64, SentinelError> {
    let p = if let Some(p) = config.gas_price(network_id) {
        debug!("using {network_id} gas price from config: {p}");
        p
    } else {
        let (msg, rx) = EthRpcMessages::get_gas_price_msg(*network_id);
        eth_rpc_tx.send(msg).await?;
        let p = rx.await??;
        debug!("using {network_id} gas price from rpc: {p}");
        p
    };
    Ok(p)
}

async fn cancel_user_ops(
    config: &SentinelConfig,
    websocket_tx: WebSocketTx,
    eth_rpc_tx: EthRpcTx,
    pk: &EthPrivateKey,
) -> Result<(), SentinelError> {
    info!("handling user op cancellation request...");

    let max_delta = config.core().max_cancellable_time_delta();
    let args = WebSocketMessagesGetCancellableUserOpArgs::new(*max_delta, vec![
        *config.native().network_id(),
        *config.host().network_id(),
    ]);

    let cancellable_user_ops = UserOps::try_from(
        call_core(
            *config.core().timeout(),
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetCancellableUserOps(Box::new(args)),
        )
        .await?,
    )?;

    if cancellable_user_ops.is_empty() {
        warn!("no user ops to cancel");
        return Ok(());
    }

    let address = pk.to_address();

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
        let (msg, rx) = EthRpcMessages::get_nonce_msg(op.destination_network_id(), address);
        eth_rpc_tx.send(msg).await?;
        let nonce = rx.await??;

        let destination_network_id = op.destination_network_id();
        let gas_price = get_gas_price(config, &destination_network_id, eth_rpc_tx.clone()).await?;
        let gas_limit = config.gas_limit(&destination_network_id)?;
        let uid = op.uid()?;
        match cancel_user_op(
            op.clone(),
            nonce,
            //native_balance,
            gas_price,
            gas_limit,
            config,
            pk,
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
    }

    Ok(())
}

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: BroadcastChannelRx,
) -> Result<UserOpCancellerBroadcastChannelMessages, SentinelError> {
    // NOTE: This loops continuously listening to the broadcasting channel, and only returns if we
    // receive a pertinent message. This way, other messages won't cause early returns in the main
    // tokios::select, so then the main_loop can continue doing its work.
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::UserOpCanceller(msg)) => break 'broadcast_channel_loop Ok(msg),
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for us
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

async fn cancellation_loop(frequency: &u64, user_op_canceller_tx: UserOpCancellerTx) -> Result<(), SentinelError> {
    // NOTE: This loop runs to send messages to the canceller loop at a configruable frequency to tell
    // it to try and cancel any cancellable user ops. It should never return, except in error.
    'cancellation_loop: loop {
        sleep(Duration::from_secs(*frequency)).await;
        warn!("{frequency}s has elapsed - sending message to cancel any cancellable user ops");
        match user_op_canceller_tx.send(UserOpCancellerMessages::CancelUserOps).await {
            Ok(_) => continue 'cancellation_loop,
            Err(e) => break 'cancellation_loop Err(e.into()),
        }
    }
}

pub async fn user_op_canceller_loop(
    mut user_op_canceller_rx: UserOpCancellerRx,
    eth_rpc_tx: EthRpcTx,
    config: SentinelConfig,
    broadcast_channel_tx: BroadcastChannelTx,
    websocket_tx: WebSocketTx,
    user_op_canceller_tx: UserOpCancellerTx,
) -> Result<(), SentinelError> {
    let name = "user op canceller";

    let mut frequency = 120; // FIXME make configurable! Make updatable whilst running too!
    let mut is_enabled = true;
    let mut core_is_connected = false;

    warn!("{name} not active yet due to no core connection");

    Env::init()?;
    let pk = Env::get_native_broadcaster_private_key()?; // FIXME: We just use the one pk now

    'user_op_canceller_loop: loop {
        tokio::select! {
            r = cancellation_loop(
                &frequency,
                user_op_canceller_tx.clone(),
            ), if (is_enabled && core_is_connected) => {
                let sleep_time = 30;
                match r {
                    Ok(_) => {
                        warn!("user op canceller cancellation loop returned Ok(()) for some reason");
                    },
                    Err(e) => {
                        error!("user op canceller cancellation loop error: {e}");
                    }
                }
                warn!("sleeping for {sleep_time}s and restarting broadcaster loop");
                sleep(Duration::from_secs(sleep_time)).await;
                continue 'user_op_canceller_loop
            },
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe()) => {
                match r {
                    Ok(msg) => {
                        let note = format!("(core is currently {}connected)", if core_is_connected { "" } else { "not "});
                        match msg {
                            UserOpCancellerBroadcastChannelMessages::Stop => {
                                warn!("msg received to stop the {name} {note}");
                                is_enabled = false;
                                continue 'user_op_canceller_loop
                            },
                            UserOpCancellerBroadcastChannelMessages::Start => {
                                warn!("msg received to start the {name} {note}");
                                is_enabled = true;
                                continue 'user_op_canceller_loop
                            },
                            UserOpCancellerBroadcastChannelMessages::CoreConnected => {
                                warn!("core connected message received in {name} {note}");
                                core_is_connected = true;
                                continue 'user_op_canceller_loop
                            },
                            UserOpCancellerBroadcastChannelMessages::CoreDisconnected => {
                                warn!("core disconnected message received in {name} {note}");
                                core_is_connected = false;
                                continue 'user_op_canceller_loop
                            },
                        }
                    },
                    Err(e) => break 'user_op_canceller_loop Err(e),
                }
            },
            r = user_op_canceller_rx.recv() , if is_enabled && core_is_connected => match r {
                Some(UserOpCancellerMessages::SetFrequency(new_frequency)) => {
                    frequency = new_frequency;
                    info!("updated user op canceller frequency to {new_frequency}");
                    continue 'user_op_canceller_loop
                },
                Some(UserOpCancellerMessages::CancelUserOps) => {
                    match cancel_user_ops(
                        &config,
                        websocket_tx.clone(),
                        eth_rpc_tx.clone(),
                        &pk,
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
                                continue 'user_op_canceller_loop
                            },
                            e => {
                                error!("unhandled user op error: {e}");
                                break 'user_op_canceller_loop Err(e.into())
                            }
                        },
                        Err(e) => {
                            error!("{e}");
                        }
                    };
                    continue 'user_op_canceller_loop
                },
                None => {
                    let m = "all {name} senders dropped!";
                    warn!("{m}");
                    break 'user_op_canceller_loop Err(SentinelError::Custom(name.into()))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'user_op_canceller_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if is_enabled { "en" } else { "dis" });
                continue 'user_op_canceller_loop
            },
        }
    }
}
