#![cfg(test)]

use std::env;

use anyhow::Result;
use dotenv::dotenv;
use jsonrpsee::ws_client::WsClient;

use crate::{check_endpoint, get_rpc_client};

const ENV_VAR: &str = "TEST_ENDPOINT";

pub async fn get_test_ws_client() -> Result<WsClient> {
    dotenv().ok();
    let time_limit = 5000; // NOTE: 5s
    let url = env::var(ENV_VAR).map_err(|_| anyhow!("Please set env var '{ENV_VAR}' to a working endpoint!"))?;
    let ws_client = get_rpc_client(&url).await?;
    check_endpoint(&ws_client, time_limit).await?;
    Ok(ws_client)
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn should_get_test_ws_client() {
        get_test_ws_client().await.unwrap();
    }
}
