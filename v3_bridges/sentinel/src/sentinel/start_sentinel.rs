use std::{result::Result, sync::Arc};

use common::BridgeSide;
use lib::{flatten_join_handle, Batch, CoreMessages, MongoMessages, ProcessorMessages, SentinelConfig, SentinelError};
use serde_json::json;
use tokio::sync::{
    mpsc,
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};

use crate::{
    cli::StartSentinelArgs,
    sentinel::{core_loop, http_server_loop, mongo_loop, processor_loop, syncer_loop},
};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(
    config: &SentinelConfig,
    sentinel_args: &StartSentinelArgs,
) -> Result<String, SentinelError> {
    let db = common_rocksdb::get_db()?;
    lib::check_init(&db)?;
    let wrapped_db = Arc::new(Mutex::new(db));

    let (processor_tx, processor_rx): (MpscTx<ProcessorMessages>, MpscRx<ProcessorMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (core_tx, core_rx): (MpscTx<CoreMessages>, MpscRx<CoreMessages>) = mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (mongo_tx, mongo_rx): (MpscTx<MongoMessages>, MpscRx<MongoMessages>) = mpsc::channel(MAX_CHANNEL_CAPACITY);

    let native_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Native, config)?,
        processor_tx.clone(),
        core_tx.clone(),
        sentinel_args.disable_native_syncer,
    ));
    let host_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Host, config)?,
        processor_tx,
        core_tx.clone(),
        sentinel_args.disable_host_syncer,
    ));

    let processor_thread = tokio::spawn(processor_loop(wrapped_db.clone(), processor_rx, mongo_tx.clone()));
    let core_thread = tokio::spawn(core_loop(wrapped_db.clone(), core_rx));
    let mongo_thread = tokio::spawn(mongo_loop(config.mongo_config.clone(), mongo_rx));
    let http_server_thread = tokio::spawn(http_server_loop(core_tx.clone(), mongo_tx.clone(), config.clone()));

    match tokio::try_join!(
        flatten_join_handle(native_syncer_thread),
        flatten_join_handle(host_syncer_thread),
        flatten_join_handle(processor_thread),
        flatten_join_handle(core_thread),
        flatten_join_handle(mongo_thread),
        flatten_join_handle(http_server_thread),
    ) {
        Ok((res_1, res_2, res_3, res_4, res_5, res_6)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "native_syncer_thread": res_1,
                "host_syncer_thread": res_2,
                "processor_thread": res_3,
                "core_thread": res_4,
                "mongo_thread": res_5,
                "http_server_thread": res_6,
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
