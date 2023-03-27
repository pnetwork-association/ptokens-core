use std::{result::Result, sync::Arc};

use common::DatabaseInterface;
use lib::{ConfigT, Heartbeats, MongoMessages, ProcessorMessages, SentinelConfig, SentinelError};
use tokio::sync::{
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};

use crate::sentinel::processor::{process_host_batch, process_native_batch};

pub async fn processor_loop<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    mut processor_rx: MpscRx<ProcessorMessages>,
    mongo_tx: MpscTx<MongoMessages>,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    info!("Starting processor loop...");

    let mut heartbeats = Heartbeats::new();
    let host_is_validating = config.host_config.is_validating();
    let native_is_validating = config.native_config.is_validating();
    let host_state_manager = config.host_config.get_state_manager();
    let native_state_manager = config.native_config.get_state_manager();

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
                            &native_state_manager,
                            &args.batch,
                            native_is_validating,
                        );
                        match result {
                            Ok(output) => {
                                let _ = args.responder.send(Ok(())); // Send an OK response so syncer can continue
                                heartbeats.push_native(&output);
                                mongo_tx.send(MongoMessages::PutNative(output)).await?;
                                mongo_tx.send(MongoMessages::PutHeartbeats(heartbeats.to_json())).await?;
                                continue 'processor_loop
                            },
                            Err(SentinelError::NoParent(e)) => {
                                debug!("native side no parent error successfully caught and returned to syncer");
                                let _ = args.responder.send(Err(SentinelError::NoParent(e)));
                                continue 'processor_loop
                            },
                            Err(SentinelError::BlockAlreadyInDb(e)) => {
                                debug!("native side block already in db successfully caught and returned to syncer");
                                let _ = args.responder.send(Err(SentinelError::BlockAlreadyInDb(e)));
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
                            &host_state_manager,
                            &args.batch,
                            host_is_validating,
                        );
                        match result {
                            Ok(output) => {
                                let _ = args.responder.send(Ok(())); // Send an OK response so syncer can continue...
                                heartbeats.push_host(&output);
                                mongo_tx.send(MongoMessages::PutHost(output)).await?;
                                mongo_tx.send(MongoMessages::PutHeartbeats(heartbeats.to_json())).await?;
                                continue 'processor_loop
                            },
                            Err(SentinelError::NoParent(e)) => {
                                debug!("host side no parent error successfully caught and returned to syncer");
                                let _ = args.responder.send(Err(SentinelError::NoParent(e)));
                                continue 'processor_loop
                            },
                            Err(SentinelError::BlockAlreadyInDb(e)) => {
                                debug!("host side block already in db successfully caught and returned to syncer");
                                let _ = args.responder.send(Err(SentinelError::BlockAlreadyInDb(e)));
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
