use std::result::Result;
/*
NOTE: can drain a channel in a loop like so:
while let Ok(value) = receiver.try_recv() {
    println!("received {}", value);
}
 */
use std::sync::Arc;

use common::DatabaseInterface;
use lib::{BroadcastMessages, ProcessorMessages, SentinelError, SyncerMessages};
use tokio::sync::{
    broadcast::{Receiver, Sender},
    Mutex,
};

use crate::sentinel::processor::{process_host_batch, process_native_batch};

pub async fn processor_loop<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    mut broadcast_rx: Receiver<BroadcastMessages>,
    mut processor_rx: Receiver<ProcessorMessages>,
    _host_syncer_tx: Sender<SyncerMessages>,
    _native_syncer_tx: Sender<SyncerMessages>,
) -> Result<(), SentinelError> {
    info!("Starting processor loop...");

    'processor_loop: loop {
        tokio::select! {
            r = processor_rx.recv() => {
                match r {
                    Ok(ProcessorMessages::ProcessNative(material)) => {
                        debug!("Processing native material...");
                        let db = guarded_db.lock().await;
                        process_native_batch(&*db, &material)?;
                        continue 'processor_loop;
                    },
                    Ok(ProcessorMessages::ProcessHost(material)) => {
                        debug!("Processing host material...");
                        let db = guarded_db.lock().await;
                        process_host_batch(&*db, &material)?;
                        continue 'processor_loop;
                    },
                    Err(e) => {
                        warn!("processor receiver error: {e}!");
                        return Err(e.into())
                    }
                }
            },
            r = broadcast_rx.recv() => {
                match r {
                    Ok(BroadcastMessages::Shutdown) => {
                        warn!("processor gracefully shutting down...");
                        return Ok::<(), SentinelError>(());
                    },
                    Err(e) => {
                        warn!("processor receiver error: {e}!");
                        return Err(e.into());
                    },
                }
            },
        }
    }
}
