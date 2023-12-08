use std::str::FromStr;

use common::{types::Result, CommonError};
use ethereum_types::{Address as EthAddress, H160, U256};
use serde::Deserialize;

use crate::{
    eth_log::{EthLog, EthLogJson, EthLogs},
    eth_receipt::{EthReceipt, EthReceipts},
    eth_receipt_type::EthReceiptType,
    eth_utils::{convert_hex_to_eth_address, convert_hex_to_h256},
};

#[derive(Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthReceiptFromJsonRpc {
    pub from: String,
    pub status: String,
    pub gas_used: String,
    pub to: Option<String>, // NOTE: Because it could be null if it's a contract creation tx.
    pub block_hash: String,
    pub logs_bloom: String,
    pub logs: Vec<EthLogJson>,
    pub block_number: String,
    pub transaction_hash: String,
    pub transaction_index: String,
    pub cumulative_gas_used: String,
    #[serde(rename = "type")]
    pub receipt_type: Option<String>,
    pub contract_address: Option<String>, // NOTE: Because it could be null if not a contract creation tx.
}

impl TryFrom<Vec<EthReceiptFromJsonRpc>> for EthReceipts {
    type Error = CommonError;

    fn try_from(v: Vec<EthReceiptFromJsonRpc>) -> std::result::Result<Self, Self::Error> {
        Ok(EthReceipts::new(
            v.iter()
                .map(EthReceipt::try_from)
                .collect::<std::result::Result<Vec<EthReceipt>, Self::Error>>()?,
        ))
    }
}

impl TryFrom<&EthReceiptFromJsonRpc> for EthReceipt {
    type Error = CommonError;

    fn try_from(json: &EthReceiptFromJsonRpc) -> std::result::Result<Self, Self::Error> {
        EthReceipt::from_json_rpc(json)
    }
}

impl EthReceipt {
    pub fn from_json_rpc(json: &EthReceiptFromJsonRpc) -> Result<Self> {
        let radix = 16;
        let logs = EthLogs::new(
            json.logs
                .iter()
                .map(EthLog::from_json)
                .collect::<Result<Vec<EthLog>>>()?,
        );
        Ok(Self {
            logs_bloom: logs.get_bloom(),
            from: convert_hex_to_eth_address(&json.from)?,
            block_hash: convert_hex_to_h256(&json.block_hash)?,
            gas_used: U256::from_str_radix(&json.gas_used, radix)?,
            block_number: U256::from_str_radix(&json.block_number, radix)?,
            transaction_hash: convert_hex_to_h256(&json.transaction_hash)?,
            transaction_index: U256::from_str_radix(&json.transaction_index, radix)?,
            cumulative_gas_used: U256::from_str_radix(&json.cumulative_gas_used, radix)?,
            to: match json.to {
                None => H160::zero(),
                Some(ref s) => convert_hex_to_eth_address(s)?,
            },
            contract_address: match json.contract_address {
                None => EthAddress::zero(),
                Some(ref s) => convert_hex_to_eth_address(s)?,
            },
            receipt_type: match json.receipt_type {
                Some(ref s) => Some(EthReceiptType::from_str(s)?),
                None => Some(EthReceiptType::Legacy),
            },
            status: matches!(json.status.as_ref(), "0x1" | "0x01"),
            logs,
        })
    }
}
