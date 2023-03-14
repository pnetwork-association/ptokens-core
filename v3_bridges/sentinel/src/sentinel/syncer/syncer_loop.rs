use std::result::Result;

use lib::{
    get_sub_mat,
    Batch,
    BroadcastMessages,
    EndpointError,
    ProcessHostArgs,
    ProcessorMessages,
    SentinelError,
    SyncerMessages,
};
use tokio::{
    sync::{broadcast::Receiver as BroadcasterRx, mpsc::Sender as MpscTx, oneshot},
    time::{sleep, Duration},
};

async fn host_loop(
    log_prefix: &str,
    mut batch: Batch,
    processor_tx: MpscTx<ProcessorMessages>,
    _host_syncer_rx: BroadcasterRx<SyncerMessages>,
) -> Result<(), SentinelError> {
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();
    let mut block_num = 16778137;

    'host_loop: loop {
        match get_sub_mat(&ws_client, block_num).await {
            Ok(block) => {
                batch.push(block);

                if !batch.is_ready_to_submit() {
                    block_num += 1;
                    continue 'host_loop;
                } else {
                    info!("{log_prefix} batch is ready to submit!");
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let args = ProcessHostArgs::new(batch.to_submission_material(), resp_tx);
                    processor_tx.send(ProcessorMessages::ProcessHost(args)).await?;
                    match resp_rx.await? {
                        Ok(_) => {
                            debug!("host oneshot channel returned ok");
                            block_num += 1;
                            batch.drain();
                            continue 'host_loop;
                        },
                        Err(SentinelError::SyncerRestart(n)) => {
                            warn!("host oneshot channel returned a syncer restart  err {n}");
                            block_num = n;
                            batch.drain();
                            continue 'host_loop;
                        },
                        Err(e) => {
                            warn!("host oneshot channel returned err {e}");
                            break 'host_loop Err(e.into());
                        },
                    }
                }
            },
            Err(SentinelError::NoBlock(_)) => {
                info!("{log_prefix} No next block yet - sleeping for {sleep_duration}ms...");
                sleep(Duration::from_millis(sleep_duration)).await;
                continue 'host_loop;
            },
            Err(e) => break Err(e),
        }
    }
}

async fn native_loop(
    log_prefix: &str,
    mut batch: Batch,
    processor_tx: MpscTx<ProcessorMessages>,
    _native_syncer_rx: BroadcasterRx<SyncerMessages>,
) -> Result<(), SentinelError> {
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();
    let mut block_num = 16778137;

    sleep(Duration::from_millis(1000000)).await; // FIXME rm!

    'native_loop: loop {
        let maybe_block = get_sub_mat(&ws_client, block_num).await;

        if let Ok(block) = maybe_block {
            batch.push(block);
            if batch.is_ready_to_submit() {
                info!("{log_prefix} Batch is ready to submit!");
                match processor_tx
                    .send(ProcessorMessages::ProcessNative(batch.to_submission_material()))
                    .await
                {
                    Ok(_) => (),
                    Err(e) => break 'native_loop Err(e.into()),
                };
                batch.drain();
                // TODO start block number is wrong!
                continue 'native_loop;
            } else {
                block_num += 1;
                continue 'native_loop;
            }
        } else if let Err(SentinelError::Endpoint(EndpointError::NoBlock(_))) = maybe_block {
            info!("{log_prefix} No next block yet - sleeping for {sleep_duration}ms...");
            sleep(Duration::from_millis(sleep_duration)).await;
            continue 'native_loop;
        } else if let Err(e) = maybe_block {
            return Err(e);
        }
    }
}

pub async fn native_syncer_loop(
    batch: Batch,
    _broadcast_rx: BroadcasterRx<BroadcastMessages>,
    syncer_rx: BroadcasterRx<SyncerMessages>,
    processor_tx: MpscTx<ProcessorMessages>,
) -> Result<(), SentinelError> {
    let log_prefix = "native_syncer:".to_string();

    tokio::select! {
        res = native_loop(&log_prefix, batch, processor_tx, syncer_rx) => res,
        _ = tokio::signal::ctrl_c() => {
            warn!("{log_prefix} shutting down...");
            Err(SentinelError::SigInt("native syncer".into()))
        },
    }
}

pub async fn host_syncer_loop(
    batch: Batch,
    _broadcast_rx: BroadcasterRx<BroadcastMessages>,
    syncer_rx: BroadcasterRx<SyncerMessages>,
    processor_tx: MpscTx<ProcessorMessages>,
) -> Result<(), SentinelError> {
    let log_prefix = "host_syncer:";

    tokio::select! {
        res = host_loop(log_prefix, batch, processor_tx, syncer_rx) => res,
        _ = tokio::signal::ctrl_c() => {
            warn!("{log_prefix} shutting down...");
            Err(SentinelError::SigInt("host syncer".into()))
        },
    }
}
