use std::result::Result;

use lib::{get_latest_block_num, Endpoints, SentinelError};
use serde_json::json;

#[derive(Debug, Subcommand)]
pub enum GetLatestBlockNumCmd {
    /// Get HOST latest block number.
    GetHostLatestBlockNum,

    /// Get NATIVE latest block number.
    GetNativeLatestBlockNum,
}

async fn get_latest_block_num_cli(endpoints: &Endpoints, is_native: bool) -> Result<String, SentinelError> {
    let block_type = if is_native { "native" } else { "host" };
    info!("Getting {block_type} latest bock number...");
    let num = get_latest_block_num(endpoints).await?;
    Ok(json!({ "jsonrpc": "2.0", "result": num }).to_string())
}

pub async fn get_native_latest_block_num(endpoints: &Endpoints) -> Result<String, SentinelError> {
    get_latest_block_num_cli(endpoints, true).await
}

pub async fn get_host_latest_block_num(endpoints: &Endpoints) -> Result<String, SentinelError> {
    get_latest_block_num_cli(endpoints, false).await
}
