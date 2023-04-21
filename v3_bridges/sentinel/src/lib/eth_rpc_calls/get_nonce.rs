use std::result::Result;

use common::strip_hex_prefix;
use ethereum_types::Address as EthAddress;
use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::{Endpoints, SentinelError};

const GET_NONCE_RPC_CMD: &str = "eth_getTransactionCount";

pub async fn get_nonce(endpoints: &Endpoints, address: &EthAddress) -> Result<u64, SentinelError> {
    let client = endpoints.get_web_socket().await?;
    let block_to_get_nonce_from = "latest";
    let nonce_hex: jsonrpsee::core::RpcResult<String> = client
        .request(GET_NONCE_RPC_CMD, rpc_params![
            format!("0x{address:x}"),
            block_to_get_nonce_from
        ])
        .await;
    match nonce_hex {
        Err(e) => Err(SentinelError::JsonRpc(e)),
        Ok(ref nonce_hex) => Ok(u64::from_str_radix(&strip_hex_prefix(nonce_hex), 16)?),
    }
}
