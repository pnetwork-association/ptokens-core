use std::result::Result;

use common::{strip_hex_prefix, BridgeSide, Byte, Bytes};
use common_eth::DefaultBlockParameter;
use ethereum_types::Address as EthAddress;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};
use serde_json::json;
use tokio::time::{sleep, Duration};

use super::{ETH_RPC_CALL_TIME_LIMIT, MAX_RPC_CALL_ATTEMPTS};
use crate::{run_timer, EndpointError, SentinelError};

const RPC_CMD: &str = "eth_call";

async fn eth_call_inner(
    to: &EthAddress,
    call_data: &[Byte],
    default_block_parameter: &DefaultBlockParameter,
    ws_client: &WsClient,
) -> Result<Bytes, SentinelError> {
    let params = json!({ "to": format!("0x{:x}", to), "data": format!("0x{}", hex::encode(call_data)) });
    let res: Result<String, jsonrpsee::core::Error> = ws_client
        .request(RPC_CMD, rpc_params![params, default_block_parameter.to_string()])
        .await;
    match res {
        Ok(ref s) => Ok(hex::decode(strip_hex_prefix(s))?),
        Err(e) => Err(SentinelError::Endpoint(EndpointError::Call(e))),
    }
}

pub async fn eth_call(
    to: &EthAddress,
    call_data: &[Byte],
    default_block_parameter: &DefaultBlockParameter,
    ws_client: &WsClient,
    sleep_time: u64,
    side: BridgeSide,
) -> Result<Bytes, SentinelError> {
    let mut attempt = 1;
    loop {
        let m = format!("making {side} eth call attempt #{attempt}");
        debug!("{m}");

        let r = tokio::select! {
            res = eth_call_inner(to, call_data, default_block_parameter, ws_client) => res,
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

    use super::{super::get_chain_id, *};
    use crate::{test_utils::get_test_ws_client, DEFAULT_SLEEP_TIME};

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_make_eth_call_inner() {
        let default_block_parameter = DefaultBlockParameter::Latest;
        let to = convert_hex_to_eth_address("0x89Ab32156e46F46D02ade3FEcbe5Fc4243B9AAeD").unwrap();
        let data = hex::decode("70a08231000000000000000000000000aeaa8c6ebb17db8056fa30a08fd3097de555f571").unwrap();
        let ws = get_test_ws_client().await;
        let chain_id = get_chain_id(&ws, DEFAULT_SLEEP_TIME, BridgeSide::default())
            .await
            .unwrap();
        if chain_id == 1 {
            // NOTE: The target for the test above is contract on ETH mainnet.
            let result = eth_call_inner(&to, &data, &default_block_parameter, &ws).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_make_eth_call() {
        let default_block_parameter = DefaultBlockParameter::Latest;
        let to = convert_hex_to_eth_address("0x89Ab32156e46F46D02ade3FEcbe5Fc4243B9AAeD").unwrap();
        let data = hex::decode("70a08231000000000000000000000000aeaa8c6ebb17db8056fa30a08fd3097de555f571").unwrap();
        let ws = get_test_ws_client().await;
        let chain_id = get_chain_id(&ws, DEFAULT_SLEEP_TIME, BridgeSide::default())
            .await
            .unwrap();
        if chain_id == 1 {
            // NOTE: The target for the test above is contract on ETH mainnet.
            let result = eth_call_inner(&to, &data, &default_block_parameter, &ws).await;
            assert!(result.is_ok());
        }
    }
}
