use std::result::Result;

use common_sentinel::{
    flatten_join_handle,
    Batch,
    ChallengeResponderMessages,
    EthRpcChannels,
    EthRpcMessages,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    StatusPublisherMessages,
    UserOpCancellerMessages,
    WebSocketMessages,
};
use futures::future::join_all;
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
    syncer::syncer_loop,
    user_op_canceller::user_op_canceller_loop,
    ws_server::ws_server_loop,
};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(config: &SentinelConfig) -> Result<String, SentinelError> {
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

    let (user_op_canceller_tx, user_op_canceller_rx): (
        MpscTx<UserOpCancellerMessages>,
        MpscRx<UserOpCancellerMessages>,
    ) = mpsc::channel(MAX_CHANNEL_CAPACITY);

    let status_thread = tokio::spawn(status_publisher_loop(
        config.clone(),
        status_rx,
        status_tx.clone(),
        broadcast_channel_tx.clone(),
        websocket_tx.clone(),
    ));

    let challenge_responder_thread = tokio::spawn(challenge_responder_loop(
        config.clone(),
        challenge_responder_rx,
        challenge_responder_tx.clone(),
        broadcast_channel_tx.clone(),
        websocket_tx.clone(),
        EthRpcSenders::from(&eth_rpc_channels),
    ));

    let user_op_canceller_thread = tokio::spawn(user_op_canceller_loop(
        user_op_canceller_rx,
        EthRpcSenders::from(&eth_rpc_channels),
        config.clone(),
        broadcast_channel_tx.clone(),
        websocket_tx.clone(),
        user_op_canceller_tx.clone(),
    ));

    let rpc_server_thread = tokio::spawn(rpc_server_loop(
        EthRpcSenders::from(&eth_rpc_channels),
        websocket_tx.clone(),
        config.clone(),
        broadcast_channel_tx.clone(),
        user_op_canceller_tx.clone(),
        status_tx.clone(),
        challenge_responder_tx.clone(),
    ));

    let ws_server_thread = tokio::spawn(ws_server_loop(
        websocket_rx,
        config.clone(),
        broadcast_channel_tx.clone(),
    ));

    let mut threads = vec![];

    let mut syncer_threads = config
        .network_ids()
        .iter()
        .map(|id| {
            Ok(tokio::spawn(syncer_loop(
                Batch::new_from_config(*id, config)?,
                config.clone(),
                EthRpcSenders::from(&eth_rpc_channels),
                websocket_tx.clone(),
                broadcast_channel_tx.clone(),
            )))
        })
        .collect::<Result<Vec<_>, SentinelError>>()?;
    threads.append(&mut syncer_threads);

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

    let mut other_threads = vec![
        ws_server_thread,
        rpc_server_thread,
        user_op_canceller_thread,
        status_thread,
        challenge_responder_thread,
    ];
    threads.append(&mut other_threads);

    match join_all(threads.into_iter().map(|t| flatten_join_handle(t)).collect::<Vec<_>>())
        .await
        .into_iter()
        .map(|x| x)
        .collect::<Result<Vec<_>, SentinelError>>()
    {
        Ok(r) => Ok(json!({ "jsonrpc": "2.0", "result": r }).to_string()),
        Err(SentinelError::SigInt(_)) => Ok(json!({ "jsonrpc": "2.0", "result": "sigint handled" }).to_string()),
        Err(e) => Err(e),
    }
}
