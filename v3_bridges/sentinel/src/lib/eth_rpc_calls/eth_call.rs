use std::result::Result;

use common::{strip_hex_prefix, Byte, Bytes};
use common_eth::DefaultBlockParameter;
use ethereum_types::Address as EthAddress;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};
use serde_json::json;

use super::ETH_RPC_CALL_TIME_LIMIT;
use crate::{run_timer, EndpointError, SentinelError};

const JSON_RPC_CMD: &str = "eth_call_inner";

async fn eth_call_inner(
    to: &EthAddress,
    call_data: &[Byte],
    default_block_parameter: &DefaultBlockParameter,
    ws_client: &WsClient,
) -> Result<Bytes, SentinelError> {
    let params = json!({ "to": format!("0x{:x}", to), "data": format!("0x{}", hex::encode(call_data)) });
    let res: Result<String, jsonrpsee::core::Error> = ws_client
        .request(JSON_RPC_CMD, rpc_params![params, default_block_parameter.to_string()])
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
) -> Result<Bytes, SentinelError> {
    let m = "making eth call".to_string();
    debug!("{m}");
    tokio::select! {
        res = eth_call_inner(to, call_data, default_block_parameter, ws_client) => res,
        _ = run_timer(ETH_RPC_CALL_TIME_LIMIT) => Err(EndpointError::TimeOut(m).into()),
        _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m).into()),
    }
}

#[cfg(test)]
mod tests {
    use common_eth::convert_hex_to_eth_address;

    use super::{super::get_chain_id, *};
    use crate::test_utils::get_test_ws_client;

    #[tokio::test]
    async fn should_make_eth_call_inner() {
        let default_block_parameter = DefaultBlockParameter::Latest;
        let to = convert_hex_to_eth_address("0x89Ab32156e46F46D02ade3FEcbe5Fc4243B9AAeD").unwrap();
        let data = hex::decode("70a08231000000000000000000000000aeaa8c6ebb17db8056fa30a08fd3097de555f571").unwrap();
        let ws = get_test_ws_client().await;
        let chain_id = get_chain_id(&ws).await.unwrap();
        if chain_id == 1 {
            // NOTE: The target for the test above is contract on ETH mainnet.
            let result = eth_call_inner(&to, &data, &default_block_parameter, &ws).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn should_make_eth_call() {
        let default_block_parameter = DefaultBlockParameter::Latest;
        let to = convert_hex_to_eth_address("0x89Ab32156e46F46D02ade3FEcbe5Fc4243B9AAeD").unwrap();
        let data = hex::decode("70a08231000000000000000000000000aeaa8c6ebb17db8056fa30a08fd3097de555f571").unwrap();
        let ws = get_test_ws_client().await;
        let chain_id = get_chain_id(&ws).await.unwrap();
        if chain_id == 1 {
            // NOTE: The target for the test above is contract on ETH mainnet.
            let result = eth_call_inner(&to, &data, &default_block_parameter, &ws).await;
            assert!(result.is_ok());
        }
    }
}
