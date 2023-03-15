use std::result::Result;

use lib::{get_sub_mat, Batch, BroadcastMessages, ProcessArgs, ProcessorMessages, SentinelError, SyncerMessages};
use tokio::{
    sync::{broadcast::Receiver as BroadcasterRx, mpsc::Sender as MpscTx, oneshot},
    time::{sleep, Duration},
};

async fn main_loop(mut batch: Batch, processor_tx: MpscTx<ProcessorMessages>) -> Result<(), SentinelError> {
    let log_prefix = format!("{} syncer", if batch.is_native() { "native" } else { "host" });
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();

    'main_loop: loop {
        match get_sub_mat(&ws_client, batch.get_block_num()).await {
            Ok(block) => {
                batch.push(block);
                if !batch.is_ready_to_submit() {
                    batch.increment_block_num();
                    continue 'main_loop;
                } else {
                    // TODO check if batch is chained correctly!
                    info!("{log_prefix} batch is ready to submit!");
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let args = ProcessArgs::new(batch.to_submission_material(), resp_tx);
                    let msg = if batch.is_native() {
                        ProcessorMessages::ProcessNative(args)
                    } else {
                        ProcessorMessages::ProcessHost(args)
                    };
                    processor_tx.send(msg).await?;
                    match resp_rx.await? {
                        Ok(_) => {
                            debug!("{log_prefix} oneshot channel returned ok");
                            batch.increment_block_num();
                        },
                        Err(SentinelError::SyncerRestart(n)) => {
                            warn!("{log_prefix} oneshot channel returned a syncer restart err {n}");
                            batch.set_block_num(n);
                        },
                        Err(e) => {
                            warn!("{log_prefix} oneshot channel returned err {e}");
                            break 'main_loop Err(e);
                        },
                    };

                    batch.drain();
                    continue 'main_loop;
                }
            },
            Err(SentinelError::NoBlock(_)) => {
                info!("{log_prefix} no next block yet - sleeping for {sleep_duration}ms...");
                sleep(Duration::from_millis(sleep_duration)).await;
                continue 'main_loop;
            },
            Err(e) => break 'main_loop Err(e),
        }
    }
}

pub async fn native_syncer_loop(
    mut batch: Batch,
    _broadcast_rx: BroadcasterRx<BroadcastMessages>,
    _syncer_rx: BroadcasterRx<SyncerMessages>,
    processor_tx: MpscTx<ProcessorMessages>,
) -> Result<(), SentinelError> {
    let block_num = 16778137; // FIXME get this from the core!
    batch.set_block_num(block_num);

    tokio::select! {
        res = main_loop(batch, processor_tx) => res,
        _ = tokio::signal::ctrl_c() => {
            warn!("native syncer shutting down...");
            Err(SentinelError::SigInt("native syncer".into()))
        },
    }
}

pub async fn host_syncer_loop(
    mut batch: Batch,
    _broadcast_rx: BroadcasterRx<BroadcastMessages>,
    _syncer_rx: BroadcasterRx<SyncerMessages>,
    processor_tx: MpscTx<ProcessorMessages>,
) -> Result<(), SentinelError> {
    let block_num = 16778137; // FIXME get this from the core!
    batch.set_block_num(block_num);

    tokio::select! {
        res = main_loop(batch, processor_tx) => res,
        _ = tokio::signal::ctrl_c() => {
            warn!("host syncer shutting down...");
            Err(SentinelError::SigInt("host syncer".into()))
        },
    }
}
