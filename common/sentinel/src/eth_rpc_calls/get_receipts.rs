use std::result::Result;

use common_eth::{EthReceipt, EthReceipts};
use common_network_ids::NetworkId;
use ethereum_types::H256 as EthHash;
use futures::{stream, Future, Stream, StreamExt};
use jsonrpsee::{
    core::{client::ClientT, Error as JsonRpseeError},
    rpc_params,
    ws_client::WsClient,
};
use serde_json::Value as JsonValue;
use tokio::time::{sleep, Duration};

use super::MAX_RPC_CALL_ATTEMPTS;
use crate::{run_timer, EndpointError, SentinelError};

const MAX_CONCURRENT_REQUESTS: usize = 250;
const RPC_CMD: &str = "eth_getTransactionReceipt";

async fn get_receipt_future<'a>(
    ws_client: &'a WsClient,
    tx_hash: &'a EthHash,
) -> impl Future<Output = Result<JsonValue, JsonRpseeError>> + 'a {
    trace!("getting receipts for tx hash: 0x{tx_hash:x}...");
    ws_client.request(RPC_CMD, rpc_params![format!("0x{tx_hash:x}")])
}

fn get_receipt_futures<'a>(
    ws_client: &'a WsClient,
    tx_hashes: &'a [EthHash],
) -> impl Stream<Item = impl Future<Output = Result<JsonValue, JsonRpseeError>> + 'a> + 'a {
    stream::iter(tx_hashes).then(|tx_hash| get_receipt_future(ws_client, tx_hash))
}

async fn get_receipts_inner(ws_client: &WsClient, tx_hashes: &[EthHash]) -> Result<EthReceipts, SentinelError> {
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

pub async fn get_receipts(
    ws_client: &WsClient,
    tx_hashes: &[EthHash],
    sleep_time: u64,
    network_id: &NetworkId,
) -> Result<EthReceipts, SentinelError> {
    const TIME_LIMIT: u64 = 10 * 1000;
    let mut attempt = 1;
    loop {
        let m = format!("{network_id} getting receipts attempt #{attempt}");
        debug!("{m}");

        let r = tokio::select! {
            res = get_receipts_inner(ws_client, tx_hashes) => res,
            _ = run_timer(TIME_LIMIT) => Err(EndpointError::TimeOut(m.clone()).into()),
            _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m.clone()).into()),
        };

        match r {
            Ok(r) => break Ok(r),
            Err(e) => match e {
                SentinelError::Endpoint(EndpointError::WsClientDisconnected(_)) => {
                    warn!("{network_id} {RPC_CMD} failed due to web socket dropping");
                    break Err(e);
                },
                _ => {
                    if attempt < MAX_RPC_CALL_ATTEMPTS {
                        attempt += 1;
                        warn!("{network_id} sleeping for {sleep_time}s before retrying...");
                        sleep(Duration::from_secs(sleep_time)).await;
                        continue;
                    } else {
                        warn!("{network_id} {RPC_CMD} failed after {attempt} attempts");
                        break Err(e);
                    }
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_block, get_latest_block_num, test_utils::get_test_ws_client, DEFAULT_SLEEP_TIME};

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_receipts_inner() {
        let ws_client = get_test_ws_client().await;
        let block_num = get_latest_block_num(&ws_client, DEFAULT_SLEEP_TIME, &NetworkId::default())
            .await
            .unwrap();
        let block = get_block(&ws_client, block_num, DEFAULT_SLEEP_TIME, &NetworkId::default())
            .await
            .unwrap();
        let tx_hashes = block.transactions;
        let result = get_receipts_inner(&ws_client, &tx_hashes).await;
        assert!(result.is_ok());
        let receipts_root = result.unwrap().get_merkle_root().unwrap();
        assert_eq!(receipts_root, block.receipts_root);
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_receipts() {
        let ws_client = get_test_ws_client().await;
        let block_num = get_latest_block_num(&ws_client, DEFAULT_SLEEP_TIME, &NetworkId::default())
            .await
            .unwrap();
        let block = get_block(&ws_client, block_num, DEFAULT_SLEEP_TIME, &NetworkId::default())
            .await
            .unwrap();
        let tx_hashes = block.transactions;
        let result = get_receipts(&ws_client, &tx_hashes, DEFAULT_SLEEP_TIME, &NetworkId::default()).await;
        assert!(result.is_ok());
        let receipts_root = result.unwrap().get_merkle_root().unwrap();
        assert_eq!(receipts_root, block.receipts_root);
    }
}
