use std::{result::Result, sync::Arc};

use common::DatabaseInterface;
use lib::{Heartbeats, MongoMessages, ProcessorMessages, SentinelConfig, SentinelError, BroadcasterMessages};
use tokio::sync::{
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};

use super::process_batch;

pub async fn processor_loop<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    mut processor_rx: MpscRx<ProcessorMessages>,
    mongo_tx: MpscTx<MongoMessages>,
    broadcaster_tx: MpscTx<BroadcasterMessages>,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    info!("Starting processor loop...");
    let mut heartbeats = Heartbeats::new();

    'processor_loop: loop {
        tokio::select! {
            r = processor_rx.recv() => {
                let db = guarded_db.lock().await;
                match r {
                    Some(ProcessorMessages::Process(args)) => {
                        let side = args.side();
                        debug!("Processing {side} material...");
                        // NOTE If we match on the process fxn call directly, we get tokio errors!
                        let result =  process_batch(
                            &*db,
                            &config.router(&side),
                            &config.state_manager(&side),
                            &args.batch,
                            config.is_validating(&side),
                            side,
                        );
                        match result {
                            Ok(output) => {
                                let _ = args.responder.send(Ok(())); // NOTE: Send an OK response so syncer can continue

                                heartbeats.push(&output);
                                mongo_tx.send(MongoMessages::PutHeartbeats(heartbeats.to_json())).await?;

                                if output.has_user_ops() {
                                    broadcaster_tx.send(BroadcasterMessages::CancelUserOps(output.user_ops())).await?;
                                }

                                continue 'processor_loop
                            },
                            Err(SentinelError::NoParent(e)) => {
                                debug!("{side} no parent error successfully caught and returned to syncer");
                                let _ = args.responder.send(Err(SentinelError::NoParent(e)));
                                continue 'processor_loop
                            },
                            Err(SentinelError::BlockAlreadyInDb(e)) => {
                                debug!("{side} block already in db successfully caught and returned to syncer");
                                let _ = args.responder.send(Err(SentinelError::BlockAlreadyInDb(e)));
                                continue 'processor_loop
                            },
                            Err(e) => {
                                warn!("{side} processor err: {e}");
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
            _ = tokio::signal::ctrl_c() => {
                warn!("processor shutting down...");
                break 'processor_loop Err(SentinelError::SigInt("processor".into()))
            },
        }
    }
}
