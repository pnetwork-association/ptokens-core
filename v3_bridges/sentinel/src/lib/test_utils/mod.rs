#![cfg(test)]

use std::env;

use anyhow::Result;
use dotenv::dotenv;
use jsonrpsee::ws_client::WsClient;

use crate::{check_endpoint, get_rpc_client};

const ENV_VAR: &str = "TEST_ENDPOINT";

pub async fn get_test_ws_client() -> Result<WsClient> {
    dotenv().ok();
    let url = env::var(ENV_VAR).map_err(|_| anyhow!("Please set env var '{ENV_VAR}' to a working endpoint!"))?;
    let ws_client = get_rpc_client(&url).await?;
    match check_endpoint(&ws_client).await {
        Ok(_) => Ok(ws_client),
        Err(_) => Err(anyhow!(
            "Endpoint check failed - check your endpoint environment variable '{ENV_VAR}'!"
        )),
    }
}
