use std::result::Result;

use common::{strip_hex_prefix, BridgeSide};
use ethereum_types::Address as EthAddress;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};
use tokio::time::{sleep, Duration};

use super::{ETH_RPC_CALL_TIME_LIMIT, MAX_RPC_CALL_ATTEMPTS};
use crate::{run_timer, EndpointError, SentinelError};

const RPC_CMD: &str = "eth_getTransactionCount";

async fn get_nonce_inner(ws_client: &WsClient, address: &EthAddress) -> Result<u64, SentinelError> {
    let block_to_get_nonce_from = "latest";
    let nonce_hex: Result<String, jsonrpsee::core::Error> = ws_client
        .request(RPC_CMD, rpc_params![format!("0x{address:x}"), block_to_get_nonce_from])
        .await;
    match nonce_hex {
        Err(e) => Err(SentinelError::JsonRpc(e)),
        Ok(ref nonce_hex) => Ok(u64::from_str_radix(&strip_hex_prefix(nonce_hex), 16)?),
    }
}

pub async fn get_nonce(
    ws_client: &WsClient,
    address: &EthAddress,
    sleep_time: u64,
    side: BridgeSide,
) -> Result<u64, SentinelError> {
    let mut attempt = 1;
    loop {
        let m = format!("{side} getting nonce for addresss {address} attempt #{attempt}");
        debug!("{m}");

        let r = tokio::select! {
            res = get_nonce_inner(ws_client, address) => res,
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
    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::{test_utils::get_test_ws_client, DEFAULT_SLEEP_TIME};

    lazy_static! {
        static ref ADDRESS: EthAddress =
            convert_hex_to_eth_address("0xedB86cd455ef3ca43f0e227e00469C3bDFA40628").unwrap();
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_latest_block_num() {
        let ws_client = get_test_ws_client().await;
        let result = get_nonce(&ws_client, &*ADDRESS, DEFAULT_SLEEP_TIME, BridgeSide::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_latest_block_num_inner() {
        let ws_client = get_test_ws_client().await;
        let result = get_nonce_inner(&ws_client, &*ADDRESS).await;
        assert!(result.is_ok());
    }
}
