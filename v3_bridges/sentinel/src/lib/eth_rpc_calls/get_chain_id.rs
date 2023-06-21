#![allow(unused)] // TODO rm once used!
use std::result::Result;

use common::strip_hex_prefix;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

use super::{ETH_RPC_CALL_TIME_LIMIT, MAX_RPC_CALL_ATTEMPTS};
use crate::{run_timer, EndpointError, SentinelError};

const RPC_CMD: &str = "eth_chainId";

async fn get_chain_id_inner(ws_client: &WsClient) -> Result<u64, SentinelError> {
    let maybe_hex: Result<String, jsonrpsee::core::Error> = ws_client.request(RPC_CMD, rpc_params![]).await;
    match maybe_hex {
        Err(e) => Err(SentinelError::JsonRpc(e)),
        Ok(ref hex) => Ok(u64::from_str_radix(&strip_hex_prefix(hex), 16)?),
    }
}

pub async fn get_chain_id(ws_client: &WsClient) -> Result<u64, SentinelError> {
    let mut attempt = 1;
    loop {
        let m = format!("getting chain id attempt #{attempt}");
        debug!("{m}");

        let r = tokio::select! {
            res = get_chain_id_inner(ws_client) => res,
            _ = run_timer(ETH_RPC_CALL_TIME_LIMIT) => Err(EndpointError::TimeOut(m.clone()).into()),
            _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m.clone()).into()),
        };

        match r {
            Ok(r) => break Ok(r),
            Err(e) => match e {
                SentinelError::Endpoint(EndpointError::WsClientDisconnected(_)) => {
                    warn!("{RPC_CMD} failed due to web socket dropping");
                    break Err(e);
                },
                _ => {
                    if attempt < MAX_RPC_CALL_ATTEMPTS {
                        attempt += 1;
                        continue;
                    } else {
                        warn!("{RPC_CMD} failed after {attempt} attempts");
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
    use crate::test_utils::get_test_ws_client;

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_chain_id_inner() {
        let ws_client = get_test_ws_client().await;
        let result = get_chain_id_inner(&ws_client).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_chain_id() {
        let ws_client = get_test_ws_client().await;
        let result = get_chain_id(&ws_client).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }
}
