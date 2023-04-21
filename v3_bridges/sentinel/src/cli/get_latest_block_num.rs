use std::result::Result;

use jsonrpsee::ws_client::WsClient;
use lib::{get_latest_block_num, SentinelError};
use serde_json::json;

#[derive(Debug, Subcommand)]
pub enum GetLatestBlockNumCmd {
    /// Get HOST latest block number.
    GetHostLatestBlockNum,

    /// Get NATIVE latest block number.
    GetNativeLatestBlockNum,
}

async fn get_latest_block_num_cli(ws_client: &WsClient, is_native: bool) -> Result<String, SentinelError> {
    let block_type = if is_native { "native" } else { "host" };
    info!("Getting {block_type} latest bock number...");
    let num = get_latest_block_num(ws_client).await?;
    Ok(json!({ "jsonrpc": "2.0", "result": num }).to_string())
}

pub async fn get_native_latest_block_num(ws_client: &WsClient) -> Result<String, SentinelError> {
    get_latest_block_num_cli(ws_client, true).await
}

pub async fn get_host_latest_block_num(ws_client: &WsClient) -> Result<String, SentinelError> {
    get_latest_block_num_cli(ws_client, false).await
}
