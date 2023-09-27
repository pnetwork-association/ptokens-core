use std::{result::Result, time::Duration};

use tokio::time::timeout;

use crate::{get_latest_block_num, Endpoints, SentinelError};

impl Endpoints {
    pub async fn check_endpoint(&self, time_limit_secs: u64) -> Result<(), SentinelError> {
        let side = self.side();
        info!("checking endpoint is working using a {time_limit_secs}s time limit...");
        let ws_client = self.get_first_ws_client().await?;
        let sleep_time = self.sleep_time();
        match timeout(
            Duration::from_secs(time_limit_secs),
            get_latest_block_num(&ws_client, sleep_time, side),
        )
        .await
        {
            Ok(_) => {
                info!("endpoint check passed!");
                Ok(())
            },
            Err(e) => {
                error!("{side} endpoint check failed");
                Err(SentinelError::from(e))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::get_test_endpoints;

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn working_endpoint_should_pass_endpoint_check() {
        let time_limit = 5000;
        let endpoints = get_test_endpoints().await;
        let result = endpoints.check_endpoint(time_limit).await;
        assert!(result.is_ok());
    }
}
