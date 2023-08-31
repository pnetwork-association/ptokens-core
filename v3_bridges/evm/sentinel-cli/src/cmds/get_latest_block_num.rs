use std::result::Result;

use common::BridgeSide;
use common_sentinel::{get_latest_block_num, SentinelError, DEFAULT_SLEEP_TIME};
use jsonrpsee::ws_client::WsClient;
use serde_json::json;

#[derive(Debug, Subcommand)]
pub enum GetLatestBlockNumCmd {
    /// Get HOST latest block number.
    GetHostLatestBlockNum,

    /// Get NATIVE latest block number.
    GetNativeLatestBlockNum,
}

async fn get_latest_block_num_cli(ws_client: &WsClient, side: BridgeSide) -> Result<String, SentinelError> {
    let block_type = if side.is_native() { "native" } else { "host" };
    info!("Getting {block_type} latest bock number...");
    let num = get_latest_block_num(ws_client, DEFAULT_SLEEP_TIME, side).await?;
    Ok(json!({ "jsonrpc": "2.0", "result": num }).to_string())
}

pub async fn get_native_latest_block_num(ws_client: &WsClient) -> Result<String, SentinelError> {
    get_latest_block_num_cli(ws_client, BridgeSide::Native).await
}

pub async fn get_host_latest_block_num(ws_client: &WsClient) -> Result<String, SentinelError> {
    get_latest_block_num_cli(ws_client, BridgeSide::Host).await
}
