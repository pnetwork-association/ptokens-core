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
    mpsc::Receiver as MpscRx,
    Mutex,
};

use crate::sentinel::processor::{process_host_batch, process_native_batch};

pub async fn processor_loop<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    _broadcast_tx: Sender<BroadcastMessages>,
    mut broadcast_rx: Receiver<BroadcastMessages>,
    mut processor_rx: MpscRx<ProcessorMessages>,
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
                    Some(ProcessorMessages::ProcessHost(args)) => {
                        debug!("Processing host material...");
                        let db = guarded_db.lock().await;
                        match process_host_batch(&*db, &args.batch) {
                            Ok(_r) => {
                                let _ = args.responder.send(Ok(())); // Send an OK response so syncer can continue...
                                continue 'processor_loop
                            },
                            Err(SentinelError::SyncerRestart(n)) => {
                                warn!("host side no parent error successfully caught and returned to syncer");
                                let _ = args.responder.send(Err(SentinelError::SyncerRestart(n)));
                                continue 'processor_loop
                            },
                            Err(e) => {
                                warn!("host processor err: {e}");
                                break 'processor_loop Err(e)
                            },
                        };
                    },





                    // TODO the rest!
                    Some(ProcessorMessages::ProcessNative(material)) => {
                        debug!("Processing native material...");
                        let db = guarded_db.lock().await;
                        match process_native_batch(&*db, &material) {
                            Ok(_r) => continue 'processor_loop, // TODO send a response via a oneshot?
                            Err(SentinelError::NoParent(e)) => {
                                warn!("native side no parent error successfully caught!");
                                break 'processor_loop Err(SentinelError::NoParent(e))
                            },
                            Err(e) => {
                                warn!("native processor err: {e}");
                                break 'processor_loop Err(e)
                            },
                        }
                    },
                    None => {
                        warn!("All processor senders dropped!");
                        break 'processor_loop Err(SentinelError::Custom("all processor senders dropped!".into()))
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
