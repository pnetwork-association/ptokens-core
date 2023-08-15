use std::result::Result;

use lib::{Batch, CoreMessages, EthRpcMessages, SentinelError};
use tokio::{
    sync::mpsc::Sender as MpscTx,
    time::{sleep, Duration},
};

async fn main_loop(
    mut batch: Batch,
    core_tx: MpscTx<CoreMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
) -> Result<(), SentinelError> {
    let side = batch.side();
    let log_prefix = format!("{} syncer", side);
    let sleep_duration = batch.get_sleep_duration();

    // NOTE: Get & set the core's latest block num into the batch...
    let (latest_block_num_msg, latest_block_num_rx) = CoreMessages::get_latest_block_num_msg(&side);
    core_tx.send(latest_block_num_msg).await?;
    batch.set_block_num(latest_block_num_rx.await?? + 1);

    // NOTE: Get & set the core's number of confs into the batch...
    let (confs_msg, confs_rx) = CoreMessages::get_confs_msg(&side);
    core_tx.send(confs_msg).await?;
    batch.set_confs(confs_rx.await??);

    'main_loop: loop {
        let (msg, rx) = EthRpcMessages::get_sub_mat_msg(side, batch.get_block_num());
        eth_rpc_tx.send(msg).await?;
        match rx.await? {
            Ok(block) => {
                batch.push(block);
                if !batch.is_ready_to_submit() {
                    batch.increment_block_num();
                    continue 'main_loop;
                } else {
                    // TODO check if batch is chained correctly!
                    info!("{log_prefix} batch is ready to submit!");
                    let (msg, rx) = CoreMessages::get_process_msg(batch.side(), batch.to_submission_material());
                    core_tx.send(msg).await?;
                    match rx.await? {
                        Ok(_) => {
                            debug!("{log_prefix} oneshot channel returned ok");
                            batch.increment_block_num();
                        },
                        Err(SentinelError::NoParent(e)) => {
                            let n = e.block_num;
                            warn!("{log_prefix} returned a no parent err for {n}!");
                            batch.drain();
                            batch.set_block_num(n - 1);
                            batch.set_single_submissions_flag();
                            continue 'main_loop;
                        },
                        Err(SentinelError::BlockAlreadyInDb(e)) => {
                            let n = e.block_num;
                            warn!("{log_prefix} block {n} already in the db!");
                            batch.drain();
                            batch.set_block_num(n + 1);
                            batch.set_single_submissions_flag();
                            continue 'main_loop;
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

pub async fn syncer_loop(
    mut batch: Batch,
    core_tx: MpscTx<CoreMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    disable: bool,
) -> Result<(), SentinelError> {
    let side = batch.side();
    let name = format!("{side} syncer");
    if disable {
        warn!("{name} disabled!")
    };
    let mut syncer_is_enabled = !disable;

    'syncer_loop: loop {
        tokio::select! {
            r = main_loop(batch.clone(), core_tx.clone(), eth_rpc_tx.clone()), if syncer_is_enabled => {
                if r.is_ok() {
                    warn!("{name} returned, restarting {name} now...");
                    continue 'syncer_loop
                } else {
                    break 'syncer_loop r
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{side} syncer shutting down...");
                break 'syncer_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if syncer_is_enabled { "en" } else { "dis" });
                continue 'syncer_loop
            },
        }
    }
}
