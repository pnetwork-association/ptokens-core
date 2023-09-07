use std::{result::Result, sync::Arc};

use common::BridgeSide;
use common_sentinel::{
    check_init,
    flatten_join_handle,
    Batch,
    BroadcasterMessages,
    CoreMessages,
    EthRpcMessages,
    MongoMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessages,
};
use serde_json::json;
use tokio::sync::{
    mpsc,
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};

use crate::{
    broadcaster::broadcaster_loop,
    core::core_loop,
    eth_rpc::eth_rpc_loop,
    mongo::mongo_loop,
    rpc_server::rpc_server_loop,
    syncer::syncer_loop,
    ws_server::ws_server_loop,
};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(
    config: &SentinelConfig,
    disable_native_syncer: bool,
    disable_host_syncer: bool,
    disable_broadcaster: bool,
    disable_rpc_server: bool,
    disable_ws_server: bool,
) -> Result<String, SentinelError> {
    let (core_tx, core_rx): (MpscTx<CoreMessages>, MpscRx<CoreMessages>) = mpsc::channel(MAX_CHANNEL_CAPACITY);
    let (websocket_tx, websocket_rx): (MpscTx<WebSocketMessages>, MpscRx<WebSocketMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);
    let (mongo_tx, mongo_rx): (MpscTx<MongoMessages>, MpscRx<MongoMessages>) = mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (native_eth_rpc_tx, native_eth_rpc_rx): (MpscTx<EthRpcMessages>, MpscRx<EthRpcMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (host_eth_rpc_tx, host_eth_rpc_rx): (MpscTx<EthRpcMessages>, MpscRx<EthRpcMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (broadcaster_tx, broadcaster_rx): (MpscTx<BroadcasterMessages>, MpscRx<BroadcasterMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let native_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Native, config)?,
        core_tx.clone(),
        native_eth_rpc_tx.clone(),
        disable_native_syncer,
    ));
    let host_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Host, config)?,
        core_tx.clone(),
        host_eth_rpc_tx.clone(),
        disable_host_syncer,
    ));

    let core_thread = tokio::spawn(core_loop(
        config.clone(),
        core_rx,
        mongo_tx.clone(),
        broadcaster_tx.clone(),
    ));
    let native_eth_rpc_thread = tokio::spawn(eth_rpc_loop(native_eth_rpc_rx, config.clone()));
    let host_eth_rpc_thread = tokio::spawn(eth_rpc_loop(host_eth_rpc_rx, config.clone()));
    let mongo_thread = tokio::spawn(mongo_loop(config.mongo().clone(), mongo_rx));
    let broadcaster_thread = tokio::spawn(broadcaster_loop(
        broadcaster_rx,
        mongo_tx.clone(),
        native_eth_rpc_tx.clone(),
        core_tx.clone(),
        config.clone(),
        disable_broadcaster,
    ));
    let rpc_server_thread = tokio::spawn(rpc_server_loop(
        core_tx.clone(),
        mongo_tx.clone(),
        websocket_tx.clone(),
        config.clone(),
        disable_rpc_server,
    ));

    let ws_server_thread = tokio::spawn(ws_server_loop(
        websocket_rx,
        core_tx.clone(),
        config.clone(),
        disable_ws_server,
    ));

    match tokio::try_join!(
        flatten_join_handle(native_syncer_thread),
        flatten_join_handle(host_syncer_thread),
        flatten_join_handle(core_thread),
        flatten_join_handle(mongo_thread),
        flatten_join_handle(rpc_server_thread),
        flatten_join_handle(native_eth_rpc_thread),
        flatten_join_handle(host_eth_rpc_thread),
        flatten_join_handle(broadcaster_thread),
        flatten_join_handle(ws_server_thread),
    ) {
        Ok((r1, r2, r3, r4, r5, r6, r7, r8, r9)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "native_syncer_thread": r1,
                "host_syncer_thread": r2,
                "core_thread": r3,
                "mongo_thread": r4,
                "rpc_server_thread": r5,
                "native_eth_rpc_thread": r6,
                "host_eth_rpc_thread": r7,
                "broadcaster_thread": r8,
                "ws_server_thread": r9,
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