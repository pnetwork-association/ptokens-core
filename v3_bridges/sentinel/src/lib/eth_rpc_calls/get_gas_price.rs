use std::result::Result;

use common::strip_hex_prefix;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

use crate::SentinelError;

const RPC_CMD: &str = "eth_gasPrice";

pub async fn get_gas_price(ws_client: &WsClient) -> Result<u64, SentinelError> {
    let maybe_hex: jsonrpsee::core::RpcResult<String> = ws_client.request(RPC_CMD, rpc_params![]).await;
    match maybe_hex {
        Err(e) => Err(SentinelError::JsonRpc(e)),
        Ok(ref hex) => Ok(u64::from_str_radix(&strip_hex_prefix(hex), 16)?),
    }
}

#[cfg(test)]
mod tests {
    use common::BridgeSide;
    use warp::{Filter, Rejection};

    use super::*;
    use crate::test_utils::get_test_ws_client;

    #[tokio::test]
    async fn should_get_gas_price() {
        let ws_client = get_test_ws_client().await;
        let result = get_gas_price(&ws_client).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }
}
