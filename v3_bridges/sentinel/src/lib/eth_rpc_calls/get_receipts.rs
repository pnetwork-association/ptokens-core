use std::result::Result;

use common_eth::{EthReceipt, EthReceipts};
use ethereum_types::H256 as EthHash;
use futures::{stream, Future, Stream, StreamExt};
use jsonrpsee::{
    core::{client::ClientT, Error as JsonRpseeError},
    rpc_params,
    ws_client::WsClient,
};
use serde_json::Value as JsonValue;

use crate::SentinelError;

const MAX_CONCURRENT_REQUESTS: usize = 250;
const GET_RECEIPT_RPC_CMD: &str = "eth_getTransactionReceipt";

async fn get_receipt_future<'a>(
    ws_client: &'a WsClient,
    tx_hash: &'a EthHash,
) -> impl Future<Output = Result<JsonValue, JsonRpseeError>> + 'a {
    trace!("[+] Getting receipts for tx hash: 0x{tx_hash:x}...");
    ws_client.request(GET_RECEIPT_RPC_CMD, rpc_params![format!("0x{tx_hash:x}")])
}

fn get_receipt_futures<'a>(
    ws_client: &'a WsClient,
    tx_hashes: &'a [EthHash],
) -> impl Stream<Item = impl Future<Output = Result<JsonValue, JsonRpseeError>> + 'a> + 'a {
    stream::iter(tx_hashes).then(|tx_hash| get_receipt_future(ws_client, tx_hash))
}

pub async fn get_receipts(ws_client: &WsClient, tx_hashes: &[EthHash]) -> Result<EthReceipts, SentinelError> {
    // TODO can I unwrap the future stream via try stream?
    // https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.4/futures/stream/trait.TryStreamExt.html
    let jsons = get_receipt_futures(ws_client, tx_hashes)
        .buffered(MAX_CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    Ok(EthReceipts::new(
        jsons
            .into_iter()
            .map(|a| Ok(EthReceipt::from_json_rpc(&serde_json::from_value(a?)?)?))
            .collect::<Result<Vec<_>, SentinelError>>()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        get_block,
        get_latest_block_num,
        test_utils::{get_test_endpoints, get_test_ws_client},
    };

    #[tokio::test]
    async fn should_get_receipts() {
        let ws_client = get_test_ws_client().await;
        let endpoints = get_test_endpoints().await;
        let block_num = get_latest_block_num(&endpoints).await.unwrap();
        let block = get_block(&ws_client, block_num).await.unwrap();
        let tx_hashes = block.transactions;
        let result = get_receipts(&ws_client, &tx_hashes).await;
        assert!(result.is_ok());
        let receipts_root = result.unwrap().get_merkle_root().unwrap();
        assert_eq!(receipts_root, block.receipts_root);
    }
}
