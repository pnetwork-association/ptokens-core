use std::result::Result;

use common::{strip_hex_prefix, Byte, Bytes};
use ethereum_types::Address as EthAddress;
use jsonrpsee::{core::client::ClientT, rpc_params};
use serde_json::json;

use crate::{endpoints::Error, Endpoints, SentinelError};

const JSON_RPC_CMD: &str = "eth_call";

// TODO parameterize the default block param (make an enum for it)

pub async fn eth_call(to: &EthAddress, call_data: &[Byte], endpoints: &Endpoints) -> Result<Bytes, SentinelError> {
    debug!("Calling read only method in contract...");
    let client = endpoints.get_rpc_client().await?;
    let params = json!({ "to": format!("0x{:x}", to), "data": format!("0x{}", hex::encode(call_data)) });
    // FIXME Could use pending to see if another sentinel has made a similar call?
    let res: jsonrpsee::core::RpcResult<String> = client.request(JSON_RPC_CMD, rpc_params![params, "latest"]).await;
    match res {
        Ok(ref s) => Ok(hex::decode(strip_hex_prefix(s))?),
        Err(e) => Err(SentinelError::Endpoint(Error::Call(e))),
    }
}

#[cfg(test)]
mod tests {
    use common_eth::convert_hex_to_eth_address;

    use super::*;
    use crate::test_utils::get_test_endpoints;

    #[tokio::test]
    async fn should_make_eth_call() {
        let to = convert_hex_to_eth_address("0x89Ab32156e46F46D02ade3FEcbe5Fc4243B9AAeD").unwrap();
        let data = hex::decode("70a08231000000000000000000000000aeaa8c6ebb17db8056fa30a08fd3097de555f571").unwrap();
        let endpoints = get_test_endpoints().await;
        let result = eth_call(&to, &data, &endpoints).await;
        assert!(result.is_ok());
    }
}
