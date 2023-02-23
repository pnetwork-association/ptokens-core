use anyhow::Result;
use common_eth::EthSubmissionMaterial;
use jsonrpsee::ws_client::WsClient;

use crate::{get_block, get_receipts};

pub async fn get_sub_mat(ws_client: &WsClient, block_num: u64) -> Result<EthSubmissionMaterial> {
    let block = get_block(ws_client, block_num).await?;
    let receipts = get_receipts(ws_client, &block.transactions).await?;
    Ok(EthSubmissionMaterial::default()
        .add_block(block)
        .and_then(|sub_mat| sub_mat.add_receipts(receipts))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_latest_block_number, test_utils::get_test_ws_client};

    #[tokio::test]
    async fn should_get_sub_mat() {
        let ws_client = get_test_ws_client().await.unwrap();
        let block_num = get_latest_block_number(&ws_client).await.unwrap();
        let result = get_sub_mat(&ws_client, block_num).await;
        assert!(result.is_ok())
    }
}
