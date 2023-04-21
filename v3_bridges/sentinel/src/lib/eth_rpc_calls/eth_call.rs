use std::result::Result;

use common::{strip_hex_prefix, Byte, Bytes};
use common_eth::DefaultBlockParameter;
use ethereum_types::Address as EthAddress;
use jsonrpsee::{core::client::ClientT, rpc_params};
use serde_json::json;

use crate::{endpoints::EndpointError, Endpoints, SentinelError};

const JSON_RPC_CMD: &str = "eth_call";

pub async fn eth_call(
    to: &EthAddress,
    call_data: &[Byte],
    default_block_parameter: &DefaultBlockParameter,
    endpoints: &Endpoints,
) -> Result<Bytes, SentinelError> {
    debug!("Calling read only method in contract...");
    let client = endpoints.get_web_socket().await?;
    let params = json!({ "to": format!("0x{:x}", to), "data": format!("0x{}", hex::encode(call_data)) });
    let res: jsonrpsee::core::RpcResult<String> = client
        .request(JSON_RPC_CMD, rpc_params![params, default_block_parameter.to_string()])
        .await;
    match res {
        Ok(ref s) => Ok(hex::decode(strip_hex_prefix(s))?),
        Err(e) => Err(SentinelError::Endpoint(EndpointError::Call(e))),
    }
}

#[cfg(test)]
mod tests {
    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::test_utils::get_test_endpoints;

    #[tokio::test]
    async fn should_make_eth_call() {
        let default_block_parameter = DefaultBlockParameter::Latest;
        let to = convert_hex_to_eth_address("0x89Ab32156e46F46D02ade3FEcbe5Fc4243B9AAeD").unwrap();
        let data = hex::decode("70a08231000000000000000000000000aeaa8c6ebb17db8056fa30a08fd3097de555f571").unwrap();
        let endpoints = get_test_endpoints().await;
        let result = eth_call(&to, &data, &default_block_parameter, &endpoints).await;
        assert!(result.is_ok());
    }
}
