use anyhow::Result;
use lib::{get_latest_block_num, get_rpc_client, EndpointsConfig};
use serde_json::json;

use crate::cli::write_file;

#[derive(Debug, Subcommand)]
pub enum GetLatestBlockNumCmd {
    /// Get HOST latest block number.
    GetHostLatestBlockNum,

    /// Get NATIVE latest block number.
    GetNativeLatestBlockNum,
}

async fn get_latest_block_num_cli(endpoints: &EndpointsConfig, is_native: bool) -> Result<String> {
    let endpoint = endpoints.get_first_endpoint(is_native)?;
    let ws_client = get_rpc_client(&endpoint).await?;
    let block_type = if is_native { "native" } else { "host" };
    info!("[+] Getting {block_type} latest bock number...");
    let num = get_latest_block_num(&ws_client).await?;
    Ok(json!({ "jsonrpc": "2.0", "result": num }).to_string())
}

pub async fn get_native_latest_block_num(endpoints: &EndpointsConfig) -> Result<String> {
    get_latest_block_num_cli(endpoints, true).await
}

pub async fn get_host_latest_block_num(endpoints: &EndpointsConfig) -> Result<String> {
    get_latest_block_num_cli(endpoints, false).await
}
