use std::{result::Result, sync::Arc};

use common::BridgeSide;
use lib::{
    flatten_join_handle,
    Batch,
    CoreAccessorMessages,
    MongoAccessorMessages,
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

use crate::sentinel::{core_accessor_loop, mongo_accessor_loop, processor_loop, syncer_loop};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(config: &SentinelConfig) -> Result<String, SentinelError> {
    let db = common_rocksdb::get_db()?;
    lib::check_init(&db)?;
    let db = Arc::new(Mutex::new(db));

    let (processor_tx, processor_rx): (MpscTx<ProcessorMessages>, MpscRx<ProcessorMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (core_tx, core_rx): (MpscTx<CoreAccessorMessages>, MpscRx<CoreAccessorMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let (mongo_tx, mongo_rx): (MpscTx<MongoAccessorMessages>, MpscRx<MongoAccessorMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);

    let native_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Native, config)?,
        processor_tx.clone(),
        core_tx.clone(),
    ));
    let host_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Host, config)?,
        processor_tx,
        core_tx.clone(),
    ));
    let processor_thread = tokio::spawn(processor_loop(db.clone(), processor_rx, mongo_tx));

    let core_accessor_thread = tokio::spawn(core_accessor_loop(db.clone(), core_rx));
    let mongo_accessor_thread = tokio::spawn(mongo_accessor_loop(mongo_rx));

    match tokio::try_join!(
        flatten_join_handle(native_syncer_thread),
        flatten_join_handle(host_syncer_thread),
        flatten_join_handle(processor_thread),
        flatten_join_handle(core_accessor_thread),
        flatten_join_handle(mongo_accessor_thread),
    ) {
        Ok((res_1, res_2, res_3, res_4, res_5)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "native_syncer_thread": res_1,
                "host_syncer_thread": res_2,
                "processor_thread": res_3,
                "core_accessor_thread": res_4,
                "mongo_accessor_thread": res_5,
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
