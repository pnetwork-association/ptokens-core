use std::result::Result;

use futures::join;
use lib::{Batch, SentinelConfig, SentinelError};
use serde_json::json;

use crate::syncer::syncer_loop;

pub async fn start_sentinel(config: &SentinelConfig) -> Result<String, SentinelError> {
    let batch_1 = Batch::new_from_config(true, &config)?;
    let batch_2 = Batch::new_from_config(false, &config)?;
    let thread_1 = tokio::spawn(async move { syncer_loop(batch_1).await });
    let thread_2 = tokio::spawn(async move { syncer_loop(batch_2).await });
    let (res_1, res_2) = join!(thread_1, thread_2);
    let thread_1_result = res_1??;
    let thread_2_result = res_2??;
    let res = json!({
        "jsonrpc": "2.0",
        "result": {
            "thread_1": thread_1_result,
            "thread_2": thread_2_result,
        },
    })
    .to_string();
    Ok(res)
}
