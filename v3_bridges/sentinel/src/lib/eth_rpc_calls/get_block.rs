use common_eth::{EthBlock, EthBlockJsonFromRpc};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

use crate::SentinelError;

const GET_FULL_TRANSACTION: bool = false;
const GET_BLOCK_BY_NUMBER_RPC_CMD: &str = "eth_getBlockByNumber";

// TODO I guess with any RPC calls we separate them out into an inner fxn, catch the error and
// sleep & retry upon that error?
// How to do endpoint roation?
//
// TODO this should take endpoint struct
pub async fn get_block(ws_client: &WsClient, block_num: u64) -> Result<EthBlock, SentinelError> {
    info!("Getting block num: {block_num}...");
    let res: jsonrpsee::core::RpcResult<EthBlockJsonFromRpc> = ws_client
        .request(GET_BLOCK_BY_NUMBER_RPC_CMD, rpc_params![
            format!("0x{block_num:x}"),
            GET_FULL_TRANSACTION
        ])
        .await;
    match res {
        Ok(ref json) => Ok(EthBlock::from_json_rpc(json)?),
        Err(jsonrpsee::core::Error::ParseError(err)) if err.to_string().contains("null") => {
            Err(SentinelError::NoBlock(block_num))
        },
        Err(err) => Err(SentinelError::JsonRpc(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        get_latest_block_num,
        test_utils::{get_test_endpoints, get_test_ws_client},
    };

    #[tokio::test]
    async fn should_get_block() {
        let ws_client = get_test_ws_client().await;
        let endpoints = get_test_endpoints().await;
        let block_num = get_latest_block_num(&endpoints).await.unwrap();
        let result = get_block(&ws_client, block_num).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_fail_to_get_block_with_correct_error() {
        let ws_client = get_test_ws_client().await;
        let block_num = i64::MAX as u64;
        match get_block(&ws_client, block_num).await {
            Err(SentinelError::NoBlock(num)) => assert_eq!(num, block_num),
            Ok(_) => panic!("Should not have succeeded!"),
            Err(e) => panic!("Wrong error received: {e}"),
        }
    }
}
