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
    _broadcast_tx: Sender<BroadcastMessages>,
    mut broadcast_rx: Receiver<BroadcastMessages>,
    mut processor_rx: Receiver<ProcessorMessages>,
    _host_syncer_tx: Sender<SyncerMessages>,
    _native_syncer_tx: Sender<SyncerMessages>,
) -> Result<(), SentinelError> {
    info!("Starting processor loop...");
    let mut i = 0;

    'processor_loop: loop {
        debug!("processor loop #{i}");
        i += 1;

        tokio::select! {
            r = processor_rx.recv() => {
                match r {
                    Ok(ProcessorMessages::ProcessNative(material)) => {
                        debug!("Processing native material...");
                        let db = guarded_db.lock().await;
                        match process_native_batch(&*db, &material) {
                            Ok(_r) => continue 'processor_loop, // TODO send a response via a oneshot?
                            Err(e) => {
                                warn!("native processor err: {e}");
                                //broadcast_tx.send(BroadcastMessages::Shutdown)?; // FIXME do we need this?
                                break 'processor_loop Err(e)
                            },
                        }
                    },
                    Ok(ProcessorMessages::ProcessHost(material)) => {
                        debug!("Processing host material...");
                        let db = guarded_db.lock().await;
                        match process_host_batch(&*db, &material) {
                            Ok(_r) => continue 'processor_loop, // TODO send res via oneshot1
                            Err(e) => {
                                warn!("host processor err: {e}");
                                //broadcast_tx.send(BroadcastMessages::Shutdown)?;
                                break 'processor_loop Err(e)
                            },
                        };
                    },
                    Err(e) => {
                        warn!("processor receiver error: {e}!");
                        break 'processor_loop Err(e.into())
                    }
                }
            },
            r = broadcast_rx.recv() => {
                match r {
                    Ok(BroadcastMessages::Shutdown) => {
                        warn!("processor gracefully shutting down...");
                        break 'processor_loop Ok::<(), SentinelError>(())
                    },
                    Err(e) => {
                        warn!("processor receiver error: {e}!");
                        break 'processor_loop Err(e.into())
                    },
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("processor shutting down...");
                break 'processor_loop Err(SentinelError::SigInt("processor".into()))
            },
        }
    }
}
