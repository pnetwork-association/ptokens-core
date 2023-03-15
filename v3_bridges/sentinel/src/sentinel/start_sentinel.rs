use std::{result::Result, sync::Arc};

use common::BridgeSide;
use lib::{
    flatten_join_handle,
    Batch,
    BroadcasterMessages,
    CoreAccessorMessages,
    ProcessorMessages,
    SentinelConfig,
    SentinelError,
    SyncerMessages,
};
use serde_json::json;
use tokio::sync::{
    broadcast,
    broadcast::{Receiver as BroadcastRx, Sender as BroadcastTx},
    mpsc,
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};

use crate::sentinel::{core_accessor_loop, processor_loop, syncer_loop};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(config: &SentinelConfig) -> Result<String, SentinelError> {
    let db = common_rocksdb::get_db()?;
    lib::check_init(&db)?;
    let wrapped_db = Arc::new(Mutex::new(db));

    // NOTE: Set up our broadcast comms for all threads...
    let (broadcast_tx_1, _): (BroadcastTx<BroadcasterMessages>, BroadcastRx<BroadcasterMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);
    let _broadcast_tx_2 = broadcast_tx_1.clone();
    let broadcast_tx_3 = broadcast_tx_1.clone();
    let broadcast_rx_3 = broadcast_tx_1.subscribe();
    let (native_syncer_tx_1, _): (BroadcastTx<SyncerMessages>, BroadcastRx<SyncerMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);
    let (host_syncer_tx_1, _): (BroadcastTx<SyncerMessages>, BroadcastRx<SyncerMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);

    let (processor_tx_1, processor_rx): (MpscTx<ProcessorMessages>, MpscRx<ProcessorMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);
    let processor_tx_2 = processor_tx_1.clone();

    let (core_accessor_tx_1, core_accessor_rx_1): (MpscTx<CoreAccessorMessages>, MpscRx<CoreAccessorMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);
    let core_accessor_tx_2 = core_accessor_tx_1.clone();

    // NOTE: Set off our threads
    let native_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Native, config)?,
        processor_tx_1,
        core_accessor_tx_1,
    ));
    let host_syncer_thread = tokio::spawn(syncer_loop(
        Batch::new_from_config(BridgeSide::Host, config)?,
        processor_tx_2,
        core_accessor_tx_2,
    ));
    let processor_thread = tokio::spawn(processor_loop(
        wrapped_db.clone(),
        broadcast_tx_3,
        broadcast_rx_3,
        processor_rx,
        native_syncer_tx_1,
        host_syncer_tx_1,
    ));

    let core_accessor_thread = tokio::spawn(core_accessor_loop(wrapped_db.clone(), core_accessor_rx_1));

    match tokio::try_join!(
        flatten_join_handle(native_syncer_thread),
        flatten_join_handle(host_syncer_thread),
        flatten_join_handle(processor_thread),
        flatten_join_handle(core_accessor_thread),
    ) {
        Ok((res_1, res_2, res_3, res_4)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": {
                "native_syncer_thread": res_1,
                "host_syncer_thread": res_2,
                "processor_thread": res_3,
                "core_accessor_thread": res_4,
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
