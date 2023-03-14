use std::{result::Result, sync::Arc};

use lib::{
    flatten_join_handle,
    Batch,
    BroadcastMessages,
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

use crate::sentinel::{
    processor_loop,
    syncer::{host_syncer_loop, native_syncer_loop},
};

const MAX_CHANNEL_CAPACITY: usize = 1337;

pub async fn start_sentinel(config: &SentinelConfig) -> Result<String, SentinelError> {
    let db = common_rocksdb::get_db()?;
    lib::check_init(&db)?;
    let wrapped_db = Arc::new(Mutex::new(db));

    // NOTE: Set up our broadcast comms for all threads...
    let (broadcast_tx_1, broadcast_rx_1): (BroadcastTx<BroadcastMessages>, BroadcastRx<BroadcastMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);
    let _broadcast_tx_2 = broadcast_tx_1.clone();
    let broadcast_tx_3 = broadcast_tx_1.clone();
    let broadcast_rx_2 = broadcast_tx_1.subscribe();
    let broadcast_rx_3 = broadcast_tx_1.subscribe();
    let (native_syncer_tx_1, native_syncer_rx): (BroadcastTx<SyncerMessages>, BroadcastRx<SyncerMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);
    let (host_syncer_tx_1, host_syncer_rx): (BroadcastTx<SyncerMessages>, BroadcastRx<SyncerMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);

    let (processor_tx_1, processor_rx): (MpscTx<ProcessorMessages>, MpscRx<ProcessorMessages>) =
        mpsc::channel(MAX_CHANNEL_CAPACITY);
    let processor_tx_2 = processor_tx_1.clone();

    let batch_1 = Batch::new_from_config(true, config)?;
    let batch_2 = Batch::new_from_config(false, config)?;

    let thread_1 = tokio::spawn(native_syncer_loop(
        batch_1,
        broadcast_rx_1,
        native_syncer_rx,
        processor_tx_1,
    ));

    let thread_2 = tokio::spawn(host_syncer_loop(
        batch_2,
        broadcast_rx_2,
        host_syncer_rx,
        processor_tx_2,
    ));

    let thread_3 = tokio::spawn(processor_loop(
        wrapped_db.clone(),
        broadcast_tx_3,
        broadcast_rx_3,
        processor_rx,
        native_syncer_tx_1,
        host_syncer_tx_1,
    ));

    match tokio::try_join!(
        flatten_join_handle(thread_1),
        flatten_join_handle(thread_2),
        flatten_join_handle(thread_3),
    ) {
        Ok((res_1, res_2, res_3)) => Ok(json!({
            "jsonrpc": "2.0",
            "result": { "thread_1": res_1, "thread_2": res_2, "thread_3": res_3},
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
