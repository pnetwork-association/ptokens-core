use std::result::Result;

use lib::{get_sub_mat, handle_sigint, Batch, BroadcastMessages, EndpointError, SentinelError};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time::{sleep, Duration},
};

async fn main_loop(log_prefix: &str, mut batch: Batch, tx: Sender<BroadcastMessages>) -> Result<String, SentinelError> {
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();

    let mut block_num = 16778137;

    'main_loop: loop {
        let maybe_block = get_sub_mat(&ws_client, block_num).await;

        if let Ok(block) = maybe_block {
            batch.push(block);
            if batch.is_ready_to_submit() {
                info!("{log_prefix} Batch is ready to submit!");
                if batch.is_native() {
                    tx.send(BroadcastMessages::ProcessNative(batch.to_submission_material()))?
                } else {
                    tx.send(BroadcastMessages::ProcessHost(batch.to_submission_material()))?
                };
                batch.drain();
                // TODO start block number is wrong!
                continue 'main_loop;
            } else {
                block_num += 1;
                continue 'main_loop;
            }
        } else if let Err(SentinelError::Endpoint(EndpointError::NoBlock(_))) = maybe_block {
            info!("{log_prefix} No next block yet - sleeping for {sleep_duration}ms...");
            sleep(Duration::from_millis(sleep_duration)).await;
            continue 'main_loop;
        } else if let Err(e) = maybe_block {
            return Err(e);
        }
    }
}

pub async fn syncer_loop(
    batch: Batch,
    tx: Sender<BroadcastMessages>,
    rx: Receiver<BroadcastMessages>,
) -> Result<String, SentinelError> {
    let syncer_type = if batch.is_native() { "native" } else { "host" };
    let log_prefix = format!("{syncer_type}_syncer:");

    tokio::select! {
        res = main_loop(&log_prefix, batch, tx) => res,
        _ = handle_sigint(&log_prefix, rx) => Ok(format!("{log_prefix} shutdown received!")),
    }
}
