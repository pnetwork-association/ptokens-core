use std::result::Result;

use common::strip_hex_prefix;
use ethereum_types::{Address as EthAddress, U256};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};
use tokio::time::{sleep, Duration};

use super::{ETH_RPC_CALL_TIME_LIMIT, MAX_RPC_CALL_ATTEMPTS};
use crate::{run_timer, EndpointError, NetworkId, SentinelError};

const RPC_CMD: &str = "eth_getBalance";

async fn get_eth_balance_inner(ws_client: &WsClient, address: &EthAddress) -> Result<U256, SentinelError> {
    let block_to_get_balance_from = "latest";
    let nonce_hex: Result<String, jsonrpsee::core::Error> = ws_client
        .request(RPC_CMD, rpc_params![
            format!("0x{address:x}"),
            block_to_get_balance_from
        ])
        .await;
    match nonce_hex {
        Err(e) => Err(SentinelError::JsonRpc(e)),
        Ok(ref nonce_hex) => Ok(U256::from_str_radix(&strip_hex_prefix(nonce_hex), 16)?),
    }
}

pub async fn get_eth_balance(
    ws_client: &WsClient,
    address: &EthAddress,
    sleep_time: u64,
    network_id: NetworkId,
) -> Result<U256, SentinelError> {
    let mut attempt = 1;
    loop {
        let m = format!("{network_id} calling {RPC_CMD} for addresss {address} attempt #{attempt}");
        debug!("{m}");

        let r = tokio::select! {
            res = get_eth_balance_inner(ws_client, address) => res,
            _ = run_timer(ETH_RPC_CALL_TIME_LIMIT) => Err(EndpointError::TimeOut(m.clone()).into()),
            _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m.clone()).into()),
        };

        match r {
            Ok(r) => break Ok(r),
            Err(e) => match e {
                SentinelError::Endpoint(EndpointError::WsClientDisconnected(_)) => {
                    warn!("{network_id} {RPC_CMD} failed due to web socket dropping");
                    break Err(e);
                },
                _ => {
                    if attempt < MAX_RPC_CALL_ATTEMPTS {
                        attempt += 1;
                        warn!("{network_id} sleeping for {sleep_time}s before retrying...");
                        sleep(Duration::from_secs(sleep_time)).await;
                        continue;
                    } else {
                        warn!("{network_id} {RPC_CMD} failed after {attempt} attempts");
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
            convert_hex_to_eth_address("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5").unwrap();
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_eth_balance() {
        let ws_client = get_test_ws_client().await;
        let result = get_eth_balance(&ws_client, &ADDRESS, DEFAULT_SLEEP_TIME, NetworkId::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_eth_balance_inner() {
        let ws_client = get_test_ws_client().await;
        let result = get_eth_balance_inner(&ws_client, &ADDRESS).await;
        assert!(result.is_ok());
    }
}
