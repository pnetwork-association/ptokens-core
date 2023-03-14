use std::result::Result;

use futures::join;
use lib::{Batch, BroadcastMessages, ProcessorMessages, SentinelConfig, SentinelError, SyncerMessages};
use serde_json::json;
use tokio::{
    signal,
    sync::{
        broadcast,
        broadcast::{Receiver, Sender},
    },
};

use crate::sentinel::{
    processor_loop,
    syncer::{host_syncer_loop, native_syncer_loop},
};

const MAX_CHANNEL_CAPACITY: usize = 1337;

use std::sync::{Arc, Mutex};
pub async fn start_sentinel(config: &SentinelConfig) -> Result<String, SentinelError> {
    let db = common_rocksdb::get_db()?;
    lib::check_init(&db)?;
    let wrapped_db = Arc::new(Mutex::new(db));

    // NOTE: Set up our broadcast comms for all threads...
    let (broadcast_tx_1, broadcast_rx_1): (Sender<BroadcastMessages>, Receiver<BroadcastMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);
    let broadcast_tx_2 = broadcast_tx_1.clone();
    let broadcast_rx_2 = broadcast_tx_1.subscribe();
    let broadcast_rx_3 = broadcast_tx_1.subscribe();

    // NOTE: Set up native syncer comms...
    let (native_syncer_tx_1, native_syncer_rx): (Sender<SyncerMessages>, Receiver<SyncerMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);

    // NOTE: Set up host syncer comms...
    let (host_syncer_tx_1, host_syncer_rx): (Sender<SyncerMessages>, Receiver<SyncerMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);

    // NOTE: Set up processor comms...
    let (processor_tx_1, processor_rx): (Sender<ProcessorMessages>, Receiver<ProcessorMessages>) =
        broadcast::channel(MAX_CHANNEL_CAPACITY);
    let processor_tx_2 = processor_tx_1.clone();

    // NOTE: Set up our batches...
    let batch_1 = Batch::new_from_config(true, config)?;
    let batch_2 = Batch::new_from_config(false, config)?;

    // NOTE: Hand everything off to async threads...
    let thread_1 =
        tokio::spawn(
            async move { native_syncer_loop(batch_1, broadcast_rx_1, native_syncer_rx, processor_tx_1).await },
        );
    let thread_2 =
        tokio::spawn(async move { host_syncer_loop(batch_2, broadcast_rx_2, host_syncer_rx, processor_tx_2).await });
    let thread_3 = tokio::spawn(async move {
        processor_loop(
            wrapped_db.clone(),
            broadcast_rx_3,
            processor_rx,
            native_syncer_tx_1,
            host_syncer_tx_1,
        )
        .await
    });

    // NOTE: Graceful shutdown upon ctrl-c...
    match signal::ctrl_c().await {
        Ok(()) => {
            warn!("Sigint caught, shutting down gracefully...");
            broadcast_tx_2.send(BroadcastMessages::Shutdown)
        },
        Err(err) => {
            warn!("Unable to listen for shutdown signal: {err} - shutting down as a precaution!");
            broadcast_tx_2.send(BroadcastMessages::Shutdown)
        },
    }?;

    let (res_1, res_2, _res_3) = join!(thread_1, thread_2, thread_3);
    let thread_1_result = res_1??;
    let thread_2_result = res_2??;

    let res = json!({
        "jsonrpc": "2.0",
        "result": {
            "thread_1": thread_1_result,
            "thread_2": thread_2_result,
        },
    })
    .to_string();
    Ok(res)
}
