use std::result::Result;

use common_sentinel::{
    flatten_join_handle,
    Batch,
    ChallengeResponderMessages,
    EthRpcChannels,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    StatusPublisherMessages,
    WebSocketMessages,
};
use futures::future::try_join_all;
use serde_json::json;
use tokio::sync::{
    broadcast,
    mpsc,
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
};

use crate::{
    challenge_responder::challenge_responder_loop,
    eth_rpc::eth_rpc_loop,
    rpc_server::rpc_server_loop,
    status_publisher::status_publisher_loop,
    syncer::syncer,
    ws_server::ws_server_loop,
};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(config: &SentinelConfig, disable: bool) -> Result<String, SentinelError> {
    let network_ids = config.network_ids();
    let eth_rpc_channels = EthRpcChannels::from(network_ids);

    let (challenge_responder_tx, challenge_responder_rx): (
        MpscTx<ChallengeResponderMessages>,
        MpscRx<ChallengeResponderMessages>,
    ) = mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (status_tx, status_rx): (MpscTx<StatusPublisherMessages>, MpscRx<StatusPublisherMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (broadcast_channel_tx, _) = broadcast::channel(MAX_CHANNEL_CAPACITY);

    let (websocket_tx, websocket_rx): (MpscTx<WebSocketMessages>, MpscRx<WebSocketMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let status_thread = tokio::spawn(status_publisher_loop(
        config.clone(),
        status_rx,
        status_tx.clone(),
        broadcast_channel_tx.clone(),
        websocket_tx.clone(),
        disable,
    ));

    let challenge_responder_thread = tokio::spawn(challenge_responder_loop(
        config.clone(),
        challenge_responder_rx,
        challenge_responder_tx.clone(),
        broadcast_channel_tx.clone(),
        websocket_tx.clone(),
        EthRpcSenders::from(&eth_rpc_channels),
        disable,
    ));

    let rpc_server_thread = tokio::spawn(rpc_server_loop(
        EthRpcSenders::from(&eth_rpc_channels),
        websocket_tx.clone(),
        config.clone(),
        broadcast_channel_tx.clone(),
        status_tx.clone(),
        challenge_responder_tx.clone(),
    ));

    let ws_server_thread = tokio::spawn(ws_server_loop(
        websocket_rx,
        config.clone(),
        broadcast_channel_tx.clone(),
    ));

    let mut threads = vec![];

    // NOTE: For each network defined in the config, a thread is created to handle a module that
    // syncs that blockchain...
    let mut syncer_threads = config
        .network_ids()
        .iter()
        .map(|id| {
            Ok(tokio::spawn(syncer(
                Batch::new_from_config(*id, config)?,
                config.clone(),
                EthRpcSenders::from(&eth_rpc_channels),
                websocket_tx.clone(),
                broadcast_channel_tx.clone(),
                disable,
            )))
        })
        .collect::<Result<Vec<_>, SentinelError>>()?;
    threads.append(&mut syncer_threads);

    // NOTE: For each network defined in the config, a thread is create dedicated to handling eth
    // RPC calls for that network...
    let mut eth_rpc_threads = eth_rpc_channels
        .to_receivers()
        .into_iter()
        .map(|(network_id, receiver)| {
            tokio::spawn(eth_rpc_loop(
                receiver,
                config.clone(),
                network_id,
                broadcast_channel_tx.clone(),
                broadcast_channel_tx.subscribe(),
            ))
        })
        .collect::<Vec<_>>();
    threads.append(&mut eth_rpc_threads);

    // NOTE: These final threads are all single modules, not dynamically generated.
    let mut other_threads = vec![
        ws_server_thread,
        rpc_server_thread,
        status_thread,
        challenge_responder_thread,
    ];
    threads.append(&mut other_threads);

    match try_join_all(threads.into_iter().map(flatten_join_handle).collect::<Vec<_>>()).await {
        Ok(r) => Ok(json!({ "jsonrpc": "2.0", "result": r }).to_string()),
        Err(SentinelError::SigInt(_)) => Ok(json!({ "jsonrpc": "2.0", "result": "sigint handled" }).to_string()),
        Err(e) => Err(e),
    }
}
