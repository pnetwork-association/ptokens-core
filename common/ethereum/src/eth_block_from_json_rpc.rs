use common::{types::Result, utils::strip_hex_prefix};
use ethereum_types::{Bloom, U256};
use serde::{Deserialize, Serialize};

use crate::{
    eth_block::EthBlock,
    eth_receipt::EthReceipts,
    eth_submission_material::EthSubmissionMaterial,
    eth_utils::{
        convert_hex_strings_to_h256s,
        convert_hex_to_bytes,
        convert_hex_to_eth_address,
        convert_hex_to_h256,
        decode_prefixed_hex,
    },
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthBlockJsonFromRpc {
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
    transactions: Vec<String>,
    base_fee_per_gas: Option<String>,
}

impl EthSubmissionMaterial {
    pub fn from_rpc(rpc_block: EthBlockJsonFromRpc) -> Result<Self> {
        EthBlock::from_json_rpc(&rpc_block).map(|block| Self {
            hash: Some(block.hash),
            eos_ref_block_num: None,
            eos_ref_block_prefix: None,
            algo_first_valid_round: None,
            receipts: EthReceipts(vec![]),
            timestamp: Some(block.timestamp),
            block_number: Some(block.number),
            parent_hash: Some(block.parent_hash),
            receipts_root: Some(block.receipts_root),
            block: Some(block),
        })
    }
}

impl EthBlock {
    pub fn from_json_rpc(json: &EthBlockJsonFromRpc) -> Result<Self> {
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
                .map(|s| convert_hex_to_h256(s))
                .collect::<Result<Vec<_>>>()?,
            base_fee_per_gas: match json.base_fee_per_gas.as_ref() {
                None => None,
                Some(hex) => Some(U256::from_str_radix(&strip_hex_prefix(hex), radix)?),
            },
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EthReceiptJsonFromRpc {
    v: String,
    r: String,
    s: String,
    to: String,
    gas: String,
    hash: String,
    from: String,
    value: String,
    input: String,
    nonce: String,
    gas_price: String,
    block_hash: String,
    block_number: String,
    status: Option<String>,
    chain_id: Option<String>,
    transaction_index: String,
    #[serde(rename = "type")]
    receipt_type: String,
}
