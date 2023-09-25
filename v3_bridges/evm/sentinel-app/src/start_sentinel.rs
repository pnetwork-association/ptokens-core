use std::result::Result;

use common::BridgeSide;
use common_sentinel::{
    flatten_join_handle,
    Batch,
    BroadcasterMessages,
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessages,
};
use serde_json::json;
use tokio::sync::{
    broadcast,
    mpsc,
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
};

use crate::{
    broadcaster::broadcaster_loop,
    eth_rpc::eth_rpc_loop,
    rpc_server::rpc_server_loop,
    syncer::syncer_loop,
    ws_server::ws_server_loop,
};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(
    config: &SentinelConfig,
    disable_broadcaster: bool,
    disable_rpc_server: bool,
    disable_ws_server: bool,
) -> Result<String, SentinelError> {
    let (broadcast_channel_tx, _) = broadcast::channel(MAX_CHANNEL_CAPACITY);

    let (websocket_tx, websocket_rx): (MpscTx<WebSocketMessages>, MpscRx<WebSocketMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (native_eth_rpc_tx, native_eth_rpc_rx): (MpscTx<EthRpcMessages>, MpscRx<EthRpcMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (host_eth_rpc_tx, host_eth_rpc_rx): (MpscTx<EthRpcMessages>, MpscRx<EthRpcMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (broadcaster_tx, broadcaster_rx): (MpscTx<BroadcasterMessages>, MpscRx<BroadcasterMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let native_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Native, config)?,
        config.clone(),
        native_eth_rpc_tx.clone(),
        websocket_tx.clone(),
        broadcast_channel_tx.clone(),
    ));
    let host_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Host, config)?,
        config.clone(),
        host_eth_rpc_tx.clone(),
        websocket_tx.clone(),
        broadcast_channel_tx.clone(),
    ));

    let native_eth_rpc_thread = tokio::spawn(eth_rpc_loop(
        native_eth_rpc_rx,
        config.clone(),
        broadcast_channel_tx.clone(),
        broadcast_channel_tx.subscribe(),
    ));
    let host_eth_rpc_thread = tokio::spawn(eth_rpc_loop(
        host_eth_rpc_rx,
        config.clone(),
        broadcast_channel_tx.clone(),
        broadcast_channel_tx.subscribe(),
    ));
    let broadcaster_thread = tokio::spawn(broadcaster_loop(
        broadcaster_rx,
        native_eth_rpc_tx.clone(),
        config.clone(),
        disable_broadcaster,
        broadcast_channel_tx.clone(),
        websocket_tx.clone(),
        broadcaster_tx.clone(),
    ));
    let rpc_server_thread = tokio::spawn(rpc_server_loop(
        host_eth_rpc_tx.clone(),
        native_eth_rpc_tx.clone(),
        websocket_tx.clone(),
        config.clone(),
        disable_rpc_server,
        broadcast_channel_tx.clone(),
        broadcaster_tx.clone(),
    ));

    let ws_server_thread = tokio::spawn(ws_server_loop(
        websocket_rx,
        config.clone(),
        disable_ws_server,
        broadcast_channel_tx.clone(),
    ));

    match tokio::try_join!(
        flatten_join_handle(native_syncer_thread),
        flatten_join_handle(host_syncer_thread),
        flatten_join_handle(rpc_server_thread),
        flatten_join_handle(native_eth_rpc_thread),
        flatten_join_handle(host_eth_rpc_thread),
        flatten_join_handle(broadcaster_thread),
        flatten_join_handle(ws_server_thread),
    ) {
        Ok((r1, r2, r3, r4, r5, r6, r7)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "native_syncer_thread": r1,
                "host_syncer_thread": r2,
                "rpc_server_thread": r3,
                "native_eth_rpc_thread": r4,
                "host_eth_rpc_thread": r5,
                "broadcaster_thread": r6,
                "ws_server_thread": r7,
            },
        })
        .to_string()),
        Err(SentinelError::SigInt(_)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": "sigint caught successfully",
        })
        .to_string()),
        Err(e) => Err(e),
    }
}
