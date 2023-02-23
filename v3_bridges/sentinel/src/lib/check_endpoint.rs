use anyhow::Result;
use jsonrpsee::ws_client::WsClient;

use crate::get_latest_block_number;

pub async fn check_endpoint(ws_client: &WsClient) -> Result<()> {
    info!("Checking endpoint is working...");
    if get_latest_block_number(ws_client).await.is_ok() {
        info!("Endpoint check passed!");
        Ok(())
    } else {
        Err(anyhow!(
            "Cannot get latest block number from endpoint - please check you config!"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_test_ws_client;

    #[tokio::test]
    async fn working_endpoint_should_pass_endpoint_check() {
        let ws_client = get_test_ws_client().await.unwrap();
        let result = check_endpoint(&ws_client).await;
        assert!(result.is_ok());
    }
}
