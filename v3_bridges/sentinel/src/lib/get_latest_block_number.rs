use anyhow::Result;
use common_eth::{EthBlockJsonFromRpc, EthSubmissionMaterial};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

use crate::constants::HEX_RADIX;

const GET_LATEST_BLOCK_NUMBER_RPC_CMD: &str = "eth_blockNumber";

pub async fn get_latest_block_number(ws_client: &WsClient) -> Result<u64> {
    info!("[+] Getting latest block number...");
    let res: jsonrpsee::core::RpcResult<String> =
        ws_client.request(GET_LATEST_BLOCK_NUMBER_RPC_CMD, rpc_params![]).await;
    match res {
        Err(err) => {
            // FIXME  This should return some error about checking the config etc
            println!("the actual error: {err}");
            Err(anyhow!("Could not get latest block number - please check your config!"))
        },
        Ok(ref s) => Ok(u64::from_str_radix(&s.replace("0x", ""), HEX_RADIX)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_test_ws_client;

    #[tokio::test]
    async fn should_get_latest_block_number() {
        let ws_client = get_test_ws_client().await.unwrap();
        let result = get_latest_block_number(&ws_client).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }
}
