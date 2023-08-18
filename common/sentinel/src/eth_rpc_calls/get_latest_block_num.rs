use std::result::Result;

use common::BridgeSide;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};
use tokio::time::{sleep, Duration};

use super::constants::{ETH_RPC_CALL_TIME_LIMIT, MAX_RPC_CALL_ATTEMPTS};
use crate::{constants::HEX_RADIX, endpoints::EndpointError, utils::run_timer, SentinelError};

const RPC_CMD: &str = "eth_blockNumber";

async fn get_latest_block_num_inner(ws_client: &WsClient) -> Result<u64, SentinelError> {
    let res: Result<String, jsonrpsee::core::Error> = ws_client.request(RPC_CMD, rpc_params![]).await;
    match res {
        Err(_) => Err(EndpointError::NoLatestBlock.into()),
        Ok(ref s) => Ok(u64::from_str_radix(&s.replace("0x", ""), HEX_RADIX)?),
    }
}

pub async fn get_latest_block_num(
    ws_client: &WsClient,
    sleep_time: u64,
    side: BridgeSide,
) -> Result<u64, SentinelError> {
    let mut attempt = 1;
    loop {
        let m = format!("{side} getting latest block num attempt #{attempt}");
        debug!("{m}");

        let r = tokio::select! {
            res = get_latest_block_num_inner(ws_client) => res,
            _ = run_timer(ETH_RPC_CALL_TIME_LIMIT) => Err(EndpointError::TimeOut(m.clone()).into()),
            _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m.clone()).into()),
        };

        match r {
            Ok(r) => break Ok(r),
            Err(e) => match e {
                SentinelError::Endpoint(EndpointError::WsClientDisconnected(_)) => {
                    warn!("{side} {RPC_CMD} failed due to web socket dropping");
                    break Err(e);
                },
                _ => {
                    if attempt < MAX_RPC_CALL_ATTEMPTS {
                        attempt += 1;
                        warn!("{side} sleeping for {sleep_time}ms before retrying...");
                        sleep(Duration::from_millis(sleep_time)).await;
                        continue;
                    } else {
                        warn!("{side} {RPC_CMD} failed after {attempt} attempts");
                        break Err(e);
                    }
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_utils::get_test_ws_client, DEFAULT_SLEEP_TIME};

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_latest_block_num() {
        let ws_client = get_test_ws_client().await;
        let result = get_latest_block_num(&ws_client, DEFAULT_SLEEP_TIME, BridgeSide::default()).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_latest_block_num_inner() {
        let ws_client = get_test_ws_client().await;
        let result = get_latest_block_num_inner(&ws_client).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }
}
