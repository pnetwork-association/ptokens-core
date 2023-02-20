use anyhow::Result;
use common_eth::{EthBlockJsonFromRpc, EthSubmissionMaterial};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

const GET_FULL_TRANSACTION: bool = false;
const GET_BLOCK_BY_NUMBER_RPC_CMD: &str = "eth_getBlockByNumber";

//TODO get the receipts as well!  we can get those concurrently since they SHOULD all be available!!

pub async fn get_block(ws_client: &WsClient, block_num: u64) -> Result<EthSubmissionMaterial> {
    info!("[+] Getting block num: {block_num}...");
    let res: EthBlockJsonFromRpc = ws_client
        .request(GET_BLOCK_BY_NUMBER_RPC_CMD, rpc_params![
            format!("0x{block_num:x}"),
            GET_FULL_TRANSACTION
        ])
        .await?;
    Ok(EthSubmissionMaterial::from_rpc(res)?)
}
