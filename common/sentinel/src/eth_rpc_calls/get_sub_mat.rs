use std::result::Result;

use common_eth::EthSubmissionMaterial;
use jsonrpsee::ws_client::WsClient;

use crate::{get_block, get_receipts, NetworkId, SentinelError};

pub async fn get_sub_mat(
    ws_client: &WsClient,
    block_num: u64,
    sleep_time: u64,
    network_id: &NetworkId,
) -> Result<EthSubmissionMaterial, SentinelError> {
    let block = get_block(ws_client, block_num, sleep_time, network_id).await?;
    let receipts = get_receipts(ws_client, &block.transactions, sleep_time, network_id).await?;
    Ok(EthSubmissionMaterial::default()
        .add_block(block)
        .and_then(|sub_mat| sub_mat.add_receipts(receipts))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_latest_block_num, test_utils::get_test_ws_client, DEFAULT_SLEEP_TIME};

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_sub_mat() {
        let ws_client = get_test_ws_client().await;
        let network_id = NetworkId::default();
        let block_num = get_latest_block_num(&ws_client, DEFAULT_SLEEP_TIME, &network_id)
            .await
            .unwrap();
        let result = get_sub_mat(&ws_client, block_num, DEFAULT_SLEEP_TIME, &network_id).await;
        assert!(result.is_ok())
    }
}
