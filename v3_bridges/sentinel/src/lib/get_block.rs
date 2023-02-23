use anyhow::Result;
use common_eth::{EthBlock, EthBlockJsonFromRpc, EthSubmissionMaterial};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

const GET_FULL_TRANSACTION: bool = false;
const GET_BLOCK_BY_NUMBER_RPC_CMD: &str = "eth_getBlockByNumber";

pub async fn get_block(ws_client: &WsClient, block_num: u64) -> Result<EthBlock> {
    info!("[+] Getting block num: {block_num}...");
    let res: jsonrpsee::core::RpcResult<EthBlockJsonFromRpc> = ws_client
        .request(GET_BLOCK_BY_NUMBER_RPC_CMD, rpc_params![
            format!("0x{block_num:x}"),
            GET_FULL_TRANSACTION
        ])
        .await;
    match res {
        Err(err) => {
            // FIXME need better error handling here to get the correct error, ie the null, and
            // return an error of our own!
            //println!("the actual error: {err}");
            Err(anyhow!("Could not get block number {block_num}!"))
        },
        Ok(ref json) => Ok(EthBlock::from_json_rpc(json)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_latest_block_number, test_utils::get_test_ws_client};

    #[tokio::test]
    async fn should_get_block() {
        let ws_client = get_test_ws_client().await.unwrap();
        let block_num = get_latest_block_number(&ws_client).await.unwrap();
        let result = get_block(&ws_client, block_num).await;
        assert!(result.is_ok());
    }
}
