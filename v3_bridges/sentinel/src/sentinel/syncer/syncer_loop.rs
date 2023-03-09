use std::result::Result;

use lib::{get_sub_mat, Batch, EndpointError, SentinelError};
use tokio::{
    sync::broadcast::Receiver,
    time::{sleep, Duration},
};

// TODO use try_recv on each loop, if Err, there are no messages from the broadcast pipeline to
// receive
// TODO use a message struct/enum with stuff in it to pass more message types.

pub async fn syncer_loop(mut batch: Batch, mut rx: Receiver<bool>) -> Result<String, SentinelError> {
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();
    let syncer_type = if batch.is_native() { "native" } else { "host" };
    let log_prefix = format!("{syncer_type}-syncer:");

    let mut block_num = 16778137;

    'main: loop {
        if let Ok(boolean) = rx.try_recv() {
            info!("{log_prefix} broadcast message received!");
            if boolean == true {
                warn!("{log_prefix} syncer shutting down!");
                return Ok(format!("{log_prefix} ctrl-c caught"));
            }
        }

        let maybe_block = get_sub_mat(&ws_client, block_num).await;

        if let Ok(block) = maybe_block {
            batch.push(block);
            if batch.is_ready_to_submit() {
                info!("{log_prefix} Batch is ready to submit!");
                break 'main;
            } else {
                block_num += 1;
                continue 'main;
            }
        } else if let Err(SentinelError::EndpointError(EndpointError::NoBlock(_))) = maybe_block {
            info!("{log_prefix} No next block yet - sleeping for {sleep_duration}ms...");
            sleep(Duration::from_millis(sleep_duration)).await;
            continue 'main;
        } else if let Err(e) = maybe_block {
            return Err(e);
        }
    }

    Ok(format!("{syncer_type}_success"))
}
