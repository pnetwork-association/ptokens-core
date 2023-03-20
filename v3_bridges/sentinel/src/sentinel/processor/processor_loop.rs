use std::{result::Result, sync::Arc};

use common::DatabaseInterface;
use lib::{Heartbeats, MongoAccessorMessages, ProcessorMessages, SentinelError};
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
    let mut heartbeats = Heartbeats::new();

    'processor_loop: loop {
        tokio::select! {
            r = processor_rx.recv() => {
                let db = guarded_db.lock().await;
                match r {
                    Some(ProcessorMessages::ProcessNative(args)) => {
                        debug!("Processing native material...");
                        // NOTE If we match on the process fxn call directly, we get tokio errors!
                        let result =  process_native_batch(
                            &*db,
                            matches!(args.is_in_sync(), Ok(true)),
                            &args.batch,
                        );
                        match result {
                            Ok(output) => {
                                let _ = args.responder.send(Ok(())); // Send an OK response so syncer can continue
                                heartbeats.push_native(&output);
                                mongo_accessor_tx.send(MongoAccessorMessages::PutNative(output)).await?;
                                mongo_accessor_tx.send(MongoAccessorMessages::PutHeartbeats(heartbeats.to_json())).await?;
                                continue 'processor_loop
                            },
                            Err(SentinelError::SyncerRestart(n)) => {
                                warn!("native side no parent error successfully caught and returned to syncer");
                                let _ = args.responder.send(Err(SentinelError::SyncerRestart(n)));
                                continue 'processor_loop
                            },
                            Err(e) => {
                                warn!("native processor err: {e}");
                                break 'processor_loop Err(e)
                            },
                        }
                    },
                    Some(ProcessorMessages::ProcessHost(args)) => {
                        debug!("Processing host material...");
                        // NOTE If we match on the process fxn call directly, we get tokio errors!
                        let result = process_host_batch(
                            &*db,
                            matches!(args.is_in_sync(), Ok(true)),
                            &args.batch
                        );
                        match result {
                            Ok(output) => {
                                let _ = args.responder.send(Ok(())); // Send an OK response so syncer can continue...
                                heartbeats.push_host(&output);
                                mongo_accessor_tx.send(MongoAccessorMessages::PutHost(output)).await?;
                                mongo_accessor_tx.send(MongoAccessorMessages::PutHeartbeats(heartbeats.to_json())).await?;
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
