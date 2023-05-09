use common_eth::{EthBlock, EthBlockJsonFromRpc};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

use super::constants::ETH_RPC_CALL_TIME_LIMIT;
use crate::{run_timer, EndpointError, SentinelError};

const GET_FULL_TRANSACTION: bool = false;
const GET_BLOCK_BY_NUMBER_RPC_CMD: &str = "eth_getBlockByNumber";

async fn get_block_inner(ws_client: &WsClient, block_num: u64) -> Result<EthBlock, SentinelError> {
    let res: Result<EthBlockJsonFromRpc, jsonrpsee::core::Error> = ws_client
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

pub async fn get_block(ws_client: &WsClient, block_num: u64) -> Result<EthBlock, SentinelError> {
    let m = format!("getting block num {block_num}");
    debug!("{m}");
    tokio::select! {
        res = get_block_inner(ws_client, block_num) => res,
        _ = run_timer(ETH_RPC_CALL_TIME_LIMIT) => Err(EndpointError::TimeOut(m).into()),
        _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_latest_block_num, test_utils::get_test_ws_client};

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_block_inner() {
        let ws_client = get_test_ws_client().await;
        let block_num = get_latest_block_num(&ws_client).await.unwrap();
        let result = get_block_inner(&ws_client, block_num).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_block() {
        let ws_client = get_test_ws_client().await;
        let block_num = get_latest_block_num(&ws_client).await.unwrap();
        let result = get_block(&ws_client, block_num).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
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
