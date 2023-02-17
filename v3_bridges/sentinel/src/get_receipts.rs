use anyhow::Result;
use common_eth::{EthReceipt, EthReceiptFromJsonRpc, EthReceipts, EthSubmissionMaterial};
use ethereum_types::H256 as EthHash;
use futures::future;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

const GET_RECEIPT_RPC_CMD: &str = "eth_getTransactionReceipt";

async fn get_receipt(ws_client: &WsClient, tx_hash: &EthHash) -> Result<EthReceipt> {
    debug!("[+] Getting receipts for tx hash: 0x{tx_hash:x}...");
    let res: EthReceiptFromJsonRpc = ws_client
        .request(GET_RECEIPT_RPC_CMD, rpc_params![format!("0x{tx_hash:x}")])
        .await?;
    Ok(EthReceipt::from_json_rpc(&res)?)
}

pub async fn get_receipts(ws_client: &WsClient, eth_sub_mat: EthSubmissionMaterial) -> Result<EthSubmissionMaterial> {
    let tx_hashes = eth_sub_mat.get_tx_hashes()?;
    let block_num = eth_sub_mat.get_block_number()?;
    info!("[+] Getting receipts for ETH block #{block_num}...");
    let futs = tx_hashes.iter().map(|hash| get_receipt(ws_client, hash));
    let receipts = future::try_join_all(futs).await?;
    info!("[+] All {} receipts retrieved!", receipts.len());
    let result = eth_sub_mat.add_receipts(EthReceipts::new(receipts))?;
    Ok(result)
}
