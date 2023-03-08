use std::result::Result;

use lib::{get_sub_mat, EndpointError, SentinelError, SubMatBatch};
use tokio::time::{sleep, Duration};

pub async fn syncer_loop(mut batch: SubMatBatch) -> Result<String, SentinelError> {
    let ws_client = batch.get_rpc_client().await?;
    let sleep_duration = batch.get_sleep_duration();
    let log_prefix = format!("{}-syncer: ", if batch.is_native() { "native" } else { "host" });

    let mut block_num = 16778137;

    'main: loop {
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

    Ok(format!("{}_success", if batch.is_native() { "native" } else { "host" }))
}
