#![cfg(test)]

use std::env;

use anyhow::Result;
use dotenv::dotenv;
use jsonrpsee::ws_client::WsClient;

use crate::get_rpc_client;

const TEST_ENDPOINT_ENV_VAR: &str = "TEST_ENDPOINT";

pub async fn get_test_ws_client() -> Result<WsClient> {
    dotenv().ok();
    let url = env::var(TEST_ENDPOINT_ENV_VAR)
        .map_err(|_| anyhow!("Please set env var '{TEST_ENDPOINT_ENV_VAR}' to a working endpoint!"))?;
    get_rpc_client(&url).await
}
