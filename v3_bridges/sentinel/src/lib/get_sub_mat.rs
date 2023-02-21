use anyhow::Result;
use common_eth::{EthBlockJsonFromRpc, EthSubmissionMaterial as EthSubMat, EthSubmissionMaterial};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

use crate::lib::{get_block::get_block, get_receipts::get_receipts};

pub async fn get_sub_mat(ws_client: &WsClient, block_num: u64) -> Result<EthSubMat> {
    let sub_mat = get_block(ws_client, block_num).await?;
    get_receipts(ws_client, sub_mat).await
}
