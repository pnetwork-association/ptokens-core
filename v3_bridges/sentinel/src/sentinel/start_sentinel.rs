use std::{result::Result, sync::Arc};

use common::BridgeSide;
use lib::{
    check_init,
    flatten_join_handle,
    Batch,
    BroadcasterMessages,
    CoreMessages,
    EthRpcMessages,
    MongoMessages,
    ProcessorMessages,
    SentinelConfig,
    SentinelError,
};
use serde_json::json;
use tokio::sync::{
    mpsc,
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};

use crate::{
    cli::StartSentinelArgs,
    sentinel::{broadcaster_loop, core_loop, eth_rpc_loop, http_server_loop, mongo_loop, processor_loop, syncer_loop},
};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(
    config: &SentinelConfig,
    sentinel_args: &StartSentinelArgs,
) -> Result<String, SentinelError> {
    let db = common_rocksdb_database::get_db_at_path(&config.get_db_path())?;
    check_init(&db)?;
    let wrapped_db = Arc::new(Mutex::new(db));

    let (processor_tx, processor_rx): (MpscTx<ProcessorMessages>, MpscRx<ProcessorMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (core_tx, core_rx): (MpscTx<CoreMessages>, MpscRx<CoreMessages>) = mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (mongo_tx, mongo_rx): (MpscTx<MongoMessages>, MpscRx<MongoMessages>) = mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (eth_rpc_tx, eth_rpc_rx): (MpscTx<EthRpcMessages>, MpscRx<EthRpcMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (broadcaster_tx, broadcaster_rx): (MpscTx<BroadcasterMessages>, MpscRx<BroadcasterMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let native_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Native, config)?,
        processor_tx.clone(),
        core_tx.clone(),
        eth_rpc_tx.clone(),
        sentinel_args.disable_native_syncer,
    ));
    let host_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Host, config)?,
        processor_tx,
        core_tx.clone(),
        eth_rpc_tx.clone(),
        sentinel_args.disable_host_syncer,
    ));

    let core_thread = tokio::spawn(core_loop(wrapped_db.clone(), core_rx));
    let eth_rpc_thread = tokio::spawn(eth_rpc_loop(eth_rpc_rx, config.clone()));
    let mongo_thread = tokio::spawn(mongo_loop(config.mongo().clone(), mongo_rx));
    let broadcaster_thread = tokio::spawn(broadcaster_loop(
        broadcaster_rx,
        mongo_tx.clone(),
        eth_rpc_tx.clone(),
        core_tx.clone(),
        config.clone(),
        sentinel_args.disable_broadcaster,
    ));
    let http_server_thread = tokio::spawn(http_server_loop(core_tx.clone(), mongo_tx.clone(), config.clone()));
    let processor_thread = tokio::spawn(processor_loop(
        wrapped_db.clone(),
        processor_rx,
        mongo_tx.clone(),
        broadcaster_tx.clone(),
        config.clone(),
    ));

    match tokio::try_join!(
        flatten_join_handle(native_syncer_thread),
        flatten_join_handle(host_syncer_thread),
        flatten_join_handle(processor_thread),
        flatten_join_handle(core_thread),
        flatten_join_handle(mongo_thread),
        flatten_join_handle(http_server_thread),
        flatten_join_handle(eth_rpc_thread),
        flatten_join_handle(broadcaster_thread),
    ) {
        Ok((r1, r2, r3, r4, r5, r6, r7, r8)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "native_syncer_thread": r1,
                "host_syncer_thread": r2,
                "processor_thread": r3,
                "core_thread": r4,
                "mongo_thread": r5,
                "http_server_thread": r6,
                "eth_rpc_thread": r7,
                "broadcaster_thread": r8,
            },
        })
        .to_string()),
        Err(SentinelError::SigInt(_)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": "sigint caught successfully",
        })
        .to_string()),
        Err(e) => {
            debug!("try_join error: {e}");
            Err(SentinelError::Json(json!({"jsonrpc": "2.0", "error": e.to_string()})))
        },
    }
}
