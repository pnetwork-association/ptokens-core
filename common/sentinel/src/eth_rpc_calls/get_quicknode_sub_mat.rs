use common::{strip_hex_prefix, CommonError};
use common_eth::{
    convert_hex_strings_to_h256s,
    convert_hex_to_bytes,
    convert_hex_to_eth_address,
    convert_hex_to_h256,
    decode_prefixed_hex,
    EthBlock,
    EthReceiptFromJsonRpc,
    EthReceiptJsonFromRpc,
    EthReceipts,
    EthSubmissionMaterial,
};
use ethereum_types::{Bloom, U256};
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use super::constants::{ETH_RPC_CALL_TIME_LIMIT, MAX_RPC_CALL_ATTEMPTS};
use crate::{run_timer, EndpointError, NetworkId, SentinelError};

const RPC_CMD: &str = "qn_getBlockWithReceipts";

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuicknodeBlockFromRpc {
    size: String,
    hash: String,
    miner: String,
    nonce: String,
    number: String,
    gas_used: String,
    mix_hash: String,
    gas_limit: String,
    timestamp: String,
    logs_bloom: String,
    extra_data: String,
    difficulty: String,
    state_root: String,
    parent_hash: String,
    sha3_uncles: String,
    uncles: Vec<String>,
    receipts_root: String,
    total_difficulty: String,
    transactions_root: String,
    base_fee_per_gas: Option<String>,
    transactions: Vec<EthReceiptJsonFromRpc>,
}

impl TryFrom<QuicknodeBlockFromRpc> for EthBlock {
    type Error = CommonError;

    fn try_from(json: QuicknodeBlockFromRpc) -> Result<Self, Self::Error> {
        let radix = 16;
        Ok(EthBlock {
            hash: convert_hex_to_h256(&json.hash)?,
            nonce: decode_prefixed_hex(&json.nonce)?,
            miner: convert_hex_to_eth_address(&json.miner)?,
            mix_hash: convert_hex_to_h256(&json.mix_hash)?,
            uncles: convert_hex_strings_to_h256s(&json.uncles)?,
            state_root: convert_hex_to_h256(&json.state_root)?,
            extra_data: convert_hex_to_bytes(&json.extra_data)?,
            parent_hash: convert_hex_to_h256(&json.parent_hash)?,
            sha3_uncles: convert_hex_to_h256(&json.sha3_uncles)?,
            receipts_root: convert_hex_to_h256(&json.receipts_root)?,
            transactions_root: convert_hex_to_h256(&json.transactions_root)?,
            size: U256::from_str_radix(&strip_hex_prefix(&json.size), radix)?,
            number: U256::from_str_radix(&strip_hex_prefix(&json.number), radix)?,
            gas_used: U256::from_str_radix(&strip_hex_prefix(&json.gas_used), radix)?,
            gas_limit: U256::from_str_radix(&strip_hex_prefix(&json.gas_limit), radix)?,
            logs_bloom: Bloom::from_slice(&convert_hex_to_bytes(&json.logs_bloom)?[..]),
            timestamp: U256::from_str_radix(&strip_hex_prefix(&json.timestamp), radix)?,
            difficulty: U256::from_str_radix(&strip_hex_prefix(&json.difficulty), radix)?,
            total_difficulty: U256::from_str_radix(&strip_hex_prefix(&json.total_difficulty), radix)?,
            transactions: json
                .transactions
                .iter()
                .map(|s| convert_hex_to_h256(s.hash()))
                .collect::<Result<Vec<_>, Self::Error>>()?,
            base_fee_per_gas: match json.base_fee_per_gas.as_ref() {
                None => None,
                Some(hex) => Some(U256::from_str_radix(&strip_hex_prefix(hex), radix)?),
            },
        })
    }
}

impl TryFrom<QuicknodeBlockAndReceiptsJson> for EthSubmissionMaterial {
    type Error = CommonError;

    fn try_from(json: QuicknodeBlockAndReceiptsJson) -> Result<Self, Self::Error> {
        let block = EthBlock::try_from(json.block)?;

        let receipts = match json.receipts {
            None => EthReceipts::default(),
            Some(r) => EthReceipts::try_from(r)?,
        };

        EthSubmissionMaterial::default()
            .add_block(block)
            .and_then(|sub_mat| sub_mat.add_receipts(receipts))
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct QuicknodeBlockAndReceiptsJson {
    block: QuicknodeBlockFromRpc,
    receipts: Option<Vec<EthReceiptFromJsonRpc>>, // NOTE: The qn rpc call can return null if no receipts.
}

async fn get_quicknode_sub_mat_inner(
    ws_client: &WsClient,
    block_num: u64,
) -> Result<EthSubmissionMaterial, SentinelError> {
    let res: Result<QuicknodeBlockAndReceiptsJson, jsonrpsee::core::Error> = ws_client
        .request(RPC_CMD, rpc_params![format!("0x{block_num:x}")])
        .await;
    match res {
        Ok(json) => EthSubmissionMaterial::try_from(json).map_err(|e| e.into()),
        Err(jsonrpsee::core::Error::ParseError(err)) if err.to_string().contains("null") => {
            Err(SentinelError::NoBlock(block_num))
        },
        Err(err) => Err(SentinelError::JsonRpc(err)),
    }
}

pub async fn get_quicknode_sub_mat(
    ws_client: &WsClient,
    block_num: u64,
    sleep_time: u64,
    network_id: &NetworkId,
) -> Result<EthSubmissionMaterial, SentinelError> {
    let mut attempt = 1;
    loop {
        let m = format!("{network_id} getting quicknode block and receipts for block {block_num} attempt #{attempt}");
        debug!("{m}");

        let r = tokio::select! {
            res = get_quicknode_sub_mat_inner(ws_client, block_num) => res,
            _ = run_timer(ETH_RPC_CALL_TIME_LIMIT) => Err(EndpointError::TimeOut(m.clone()).into()),
            _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m.clone()).into()),
        };

        match r {
            Ok(r) => break Ok(r),
            Err(e) => match e {
                SentinelError::Endpoint(EndpointError::WsClientDisconnected(_)) => {
                    warn!("{network_id} {RPC_CMD} failed due to web socket dropping");
                    break Err(e);
                },
                other_error => {
                    error!("{other_error}");
                    if other_error
                        .to_string()
                        .contains("the method qn_getBlockWithReceipts does not exist")
                    {
                        debug!("quicknode rpc methods not available");
                        // NOTE: No point retrying if the method isn't available, so we fail fast instead.
                        break Err(SentinelError::QuicknodeNotAvailable);
                    } else if attempt < MAX_RPC_CALL_ATTEMPTS {
                        attempt += 1;
                        warn!("{network_id} sleeping for {sleep_time}s before retrying...");
                        sleep(Duration::from_secs(sleep_time)).await;
                        continue;
                    } else {
                        warn!("{network_id} {RPC_CMD} failed after {attempt} attempts");
                        break Err(other_error);
                    }
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common_chain_ids::EthChainId;

    use super::*;
    use crate::{get_latest_block_num, test_utils::get_test_ws_client, DEFAULT_SLEEP_TIME};

    #[tokio::test]
    #[cfg_attr(not(feature = "test-eth-rpc"), ignore)]
    async fn should_get_sub_mat_via_quicknode() {
        let ws_client = get_test_ws_client().await;
        let network_id = NetworkId::from_str("polygon").unwrap();
        let eth_chain_id = EthChainId::try_from(network_id).unwrap();
        let block_num = get_latest_block_num(&ws_client, DEFAULT_SLEEP_TIME, &network_id)
            .await
            .unwrap()
            - 10; // NOTE: Sometimes quicknode reports a latest block num that it can't
                  // yet get the full block for.
        let result = get_quicknode_sub_mat_inner(&ws_client, block_num).await;
        assert!(result.is_ok());
    }
}
