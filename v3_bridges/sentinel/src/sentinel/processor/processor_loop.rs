use std::{result::Result, sync::Arc};

use common::DatabaseInterface;
use lib::{MongoAccessorMessages, ProcessorMessages, SentinelError};
use tokio::sync::{
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};

use crate::sentinel::processor::{process_host_batch, process_native_batch};

pub async fn processor_loop<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    mut processor_rx: MpscRx<ProcessorMessages>,
    mongo_accessor_tx: MpscTx<MongoAccessorMessages>,
) -> Result<(), SentinelError> {
    info!("Starting processor loop...");
    let mut i = 0;

    'processor_loop: loop {
        debug!("processor loop #{i}");
        i += 1;

        tokio::select! {
            r = processor_rx.recv() => {
                match r {
                    Some(ProcessorMessages::ProcessNative(args)) => {
                        debug!("Processing native material...");
                        let db = guarded_db.lock().await;
                        match process_native_batch(&*db, &args.batch, mongo_accessor_tx.clone()) {
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
                        }
                    },
                    Some(ProcessorMessages::ProcessHost(args)) => {
                        debug!("Processing host material...");
                        let db = guarded_db.lock().await;
                        match process_host_batch(&*db, &args.batch, mongo_accessor_tx.clone()) {
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
                    None => {
                        warn!("All processor senders dropped!");
                        break 'processor_loop Err(SentinelError::Custom("all processor senders dropped!".into()))
                    }
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("processor shutting down...");
                break 'processor_loop Err(SentinelError::SigInt("processor".into()))
            },
        }
    }
}
