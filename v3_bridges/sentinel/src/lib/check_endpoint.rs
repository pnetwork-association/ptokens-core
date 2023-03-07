use std::{result::Result, time::Duration};

use jsonrpsee::ws_client::WsClient;
use tokio::time::timeout;

use crate::{get_latest_block_num, SentinelError};

pub async fn check_endpoint(ws_client: &WsClient, time_limit: u64) -> Result<(), SentinelError> {
    info!("Checking endpoint is working using a {time_limit}ms time limit...");
    match timeout(Duration::from_millis(time_limit), get_latest_block_num(ws_client)).await {
        Ok(_) => {
            info!("Endpoint check passed!");
            Ok(())
        },
        Err(e) => Err(SentinelError::from(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_test_ws_client;

    #[tokio::test]
    async fn working_endpoint_should_pass_endpoint_check() {
        let time_limit = 5000;
        let ws_client = get_test_ws_client().await;
        let result = check_endpoint(&ws_client, time_limit).await;
        assert!(result.is_ok());
    }
}
