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

async fn main_loop(
    mut batch: Batch,
    processor_tx: MpscTx<ProcessorMessages>,
    _host_syncer_rx: BroadcasterRx<SyncerMessages>,
) -> Result<(), SentinelError> {
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
                    let args = ProcessHostArgs::new(batch.to_submission_material(), resp_tx);
                    processor_tx.send(ProcessorMessages::ProcessHost(args)).await?;
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

async fn native_loop(
    log_prefix: &str,
    mut batch: Batch,
    processor_tx: MpscTx<ProcessorMessages>,
    _native_syncer_rx: BroadcasterRx<SyncerMessages>,
) -> Result<(), SentinelError> {
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();

    sleep(Duration::from_millis(1000000)).await; // FIXME rm!

    'native_loop: loop {
        let maybe_block = get_sub_mat(&ws_client, batch.get_block_num()).await;

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
                batch.increment_block_num();
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
    mut batch: Batch,
    _broadcast_rx: BroadcasterRx<BroadcastMessages>,
    syncer_rx: BroadcasterRx<SyncerMessages>,
    processor_tx: MpscTx<ProcessorMessages>,
) -> Result<(), SentinelError> {
    let block_num = 16778137; // FIXME get this from the core!
    let log_prefix = "native_syncer:".to_string();
    batch.set_block_num(block_num);

    tokio::select! {
        res = native_loop(&log_prefix, batch, processor_tx, syncer_rx) => res,
        _ = tokio::signal::ctrl_c() => {
            warn!("{log_prefix} shutting down...");
            Err(SentinelError::SigInt("native syncer".into()))
        },
    }
}

pub async fn host_syncer_loop(
    mut batch: Batch,
    _broadcast_rx: BroadcasterRx<BroadcastMessages>,
    syncer_rx: BroadcasterRx<SyncerMessages>,
    processor_tx: MpscTx<ProcessorMessages>,
) -> Result<(), SentinelError> {
    let block_num = 16778137; // FIXME get this from the core!
    let log_prefix = "host_syncer:";
    batch.set_block_num(block_num);

    tokio::select! {
        res = main_loop(batch, processor_tx, syncer_rx) => res,
        _ = tokio::signal::ctrl_c() => {
            warn!("{log_prefix} shutting down...");
            Err(SentinelError::SigInt("host syncer".into()))
        },
    }
}
