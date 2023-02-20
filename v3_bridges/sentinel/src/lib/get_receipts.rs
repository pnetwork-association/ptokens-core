use anyhow::Result;
use common_eth::{EthReceipt, EthReceipts, EthSubmissionMaterial};
use ethereum_types::H256 as EthHash;
use futures::{stream, Future, Stream, StreamExt};
use jsonrpsee::{
    core::{client::ClientT, Error as JsonRpseeError},
    rpc_params,
    ws_client::WsClient,
};
use serde_json::Value as JsonValue;

const MAX_CONCURRENT_REQUESTS: usize = 250;
const GET_RECEIPT_RPC_CMD: &str = "eth_getTransactionReceipt";

async fn get_receipt_future<'a>(
    ws_client: &'a WsClient,
    tx_hash: &'a EthHash,
) -> impl Future<Output = std::result::Result<JsonValue, JsonRpseeError>> + 'a {
    debug!("[+] Getting receipts for tx hash: 0x{tx_hash:x}...");
    ws_client.request(GET_RECEIPT_RPC_CMD, rpc_params![format!("0x{tx_hash:x}")])
}

fn get_receipt_futures<'a>(
    ws_client: &'a WsClient,
    tx_hashes: &'a [EthHash],
) -> impl Stream<Item = impl Future<Output = Result<JsonValue, JsonRpseeError>> + 'a> + 'a {
    stream::iter(tx_hashes).then(|tx_hash| get_receipt_future(ws_client, tx_hash))
}

pub async fn get_receipts(ws_client: &WsClient, eth_sub_mat: EthSubmissionMaterial) -> Result<EthSubmissionMaterial> {
    // TODO can I unwrap the future stream via try stream?
    // https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.4/futures/stream/trait.TryStreamExt.html
    let jsons = get_receipt_futures(ws_client, &eth_sub_mat.get_tx_hashes()?)
        .buffered(MAX_CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    let receipts = EthReceipts::new(
        jsons
            .into_iter()
            .map(|a| Ok(EthReceipt::from_json_rpc(&serde_json::from_value(a?)?)?))
            .collect::<Result<Vec<_>>>()?,
    );

    let result = eth_sub_mat.add_receipts(receipts)?;
    Ok(result)
}
