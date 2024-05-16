use common_sentinel::{
    BroadcastChannelMessages,
    EthRpcSenders,
    RpcServerBroadcastChannelMessages,
    SentinelConfig,
    SentinelError,
};
use warp::Filter;

use super::{JsonRpcRequest, RpcCalls};
use crate::type_aliases::{
    BroadcastChannelRx,
    BroadcastChannelTx,
    ChallengeResponderTx,
    StatusPublisherTx,
    WebSocketTx,
};

async fn start_rpc_server(
    eth_rpc_senders: EthRpcSenders,
    websocket_tx: WebSocketTx,
    config: SentinelConfig,
    broadcast_channel_tx: BroadcastChannelTx,
    core_cxn: bool,
    status_tx: StatusPublisherTx,
    challenge_responder_tx: ChallengeResponderTx,
) -> Result<(), SentinelError> {
    debug!("rpc server listening!");
    let core_cxn_filter = warp::any().map(move || core_cxn);
    let status_tx_filter = warp::any().map(move || status_tx.clone());
    let websocket_tx_filter = warp::any().map(move || websocket_tx.clone());
    let eth_rpc_senders_filter = warp::any().map(move || eth_rpc_senders.clone());
    let broadcast_channel_tx_filter = warp::any().map(move || broadcast_channel_tx.clone());
    let challenge_responder_tx_filter = warp::any().map(move || challenge_responder_tx.clone());

    let rpc = warp::path("v1")
        .and(warp::path("rpc"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16)) // FIXME make configurable
        .and(warp::body::json::<JsonRpcRequest>())
        .and(warp::any().map(move || config.clone()))
        .and(websocket_tx_filter.clone())
        .and(eth_rpc_senders_filter.clone())
        .and(broadcast_channel_tx_filter.clone())
        .and(status_tx_filter.clone())
        .and(challenge_responder_tx_filter.clone())
        .and(core_cxn_filter)
        .map(RpcCalls::new)
        .and_then(|r: RpcCalls| async move { r.handle().await });

    warp::serve(rpc).run(([127, 0, 0, 1], 3030)).await; // FIXME make configurable

    Ok(())
}

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: BroadcastChannelRx,
    _core_connection: bool,
) -> Result<RpcServerBroadcastChannelMessages, SentinelError> {
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::RpcServer(msg)) => {
                // NOTE: We have a pertinent message, break and send it to the rpc_server_loop...
                break 'broadcast_channel_loop Ok(msg);
            },
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for the syncer
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

pub async fn rpc_server_loop(
    eth_rpc_senders: EthRpcSenders,
    websocket_tx: WebSocketTx,
    config: SentinelConfig,
    broadcast_channel_tx: BroadcastChannelTx,
    status_tx: StatusPublisherTx,
    challenge_responder_tx: ChallengeResponderTx,
) -> Result<(), SentinelError> {
    let name = "rpc server";

    let rpc_server_is_enabled = true; // FIXME rm
    let mut core_connection_status = false;

    'rpc_server_loop: loop {
        tokio::select! {
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe(), core_connection_status) => {
                match r {
                    Ok(RpcServerBroadcastChannelMessages::CoreConnected) => {
                        core_connection_status = true;
                        continue 'rpc_server_loop
                    },
                    Ok(RpcServerBroadcastChannelMessages::CoreDisconnected) => {
                        core_connection_status = false;
                        continue 'rpc_server_loop
                    },
                    Err(e) => break 'rpc_server_loop Err(e),
                }
            },
            r = start_rpc_server(
                eth_rpc_senders.clone(),
                websocket_tx.clone(),
                config.clone(),
                broadcast_channel_tx.clone(),
                core_connection_status,
                status_tx.clone(),
                challenge_responder_tx.clone(),
            ), if rpc_server_is_enabled => {
                if r.is_ok() {
                    warn!("{name} returned, restarting {name} now...");
                    continue 'rpc_server_loop
                } else {
                    break 'rpc_server_loop r
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'rpc_server_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if rpc_server_is_enabled { "en" } else { "dis" });
                continue 'rpc_server_loop
            }
        }
    }
}
