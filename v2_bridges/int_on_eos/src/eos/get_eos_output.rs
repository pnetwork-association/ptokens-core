use std::time::{SystemTime, UNIX_EPOCH};

use common::{traits::DatabaseInterface, types::Result};
use common_eos::EosState;
use common_eth::{EthDbUtils, EthDbUtilsExt, EthTransaction, EthTransactions, EthTxInfoCompatible};
use serde::{Deserialize, Serialize};

use crate::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EosOutput {
    pub eos_latest_block_number: u64,
    pub int_signed_transactions: Vec<TxInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub int_tx_hash: String,
    pub int_tx_amount: String,
    pub eos_tx_amount: String,
    pub int_account_nonce: u64,
    pub int_tx_recipient: String,
    pub witnessed_timestamp: u64,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub destination_chain_id: String,
    pub native_token_address: String,
    pub int_signed_tx: Option<String>,
    pub int_latest_block_number: usize,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

impl TxInfo {
    pub fn new<T: EthTxInfoCompatible>(
        tx: &T,
        tx_info: &IntOnEosIntTxInfo,
        nonce: u64,
        int_latest_block_number: usize,
    ) -> Result<TxInfo> {
        Ok(TxInfo {
            int_latest_block_number,
            broadcast: false,
            broadcast_tx_hash: None,
            int_account_nonce: nonce,
            broadcast_timestamp: None,
            int_signed_tx: tx.eth_tx_hex(),
            _id: format!("pint-on-eos-int-{}", nonce),
            int_tx_amount: tx_info.amount.to_string(),
            int_tx_hash: format!("0x{}", tx.get_tx_hash()),
            eos_tx_amount: tx_info.eos_tx_amount.to_string(),
            int_tx_recipient: tx_info.destination_address.clone(),
            originating_address: tx_info.origin_address.to_string(),
            host_token_address: tx_info.eos_token_address.to_string(),
            originating_tx_hash: tx_info.originating_tx_id.to_string(),
            destination_chain_id: tx_info.destination_chain_id.to_hex()?,
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            native_token_address: format!("0x{}", hex::encode(tx_info.int_token_address)),
        })
    }
}

pub fn get_tx_infos_from_signed_txs(
    txs: &[EthTransaction],
    tx_infos: &IntOnEosIntTxInfos,
    int_account_nonce: u64,
    int_latest_block_number: usize,
) -> Result<Vec<TxInfo>> {
    info!("✔ Getting output from INT txs...");
    let number_of_txs = txs.len() as u64;
    let start_nonce = if number_of_txs > int_account_nonce {
        return Err("INT account nonce has not been incremented correctly!".into());
    } else {
        int_account_nonce - number_of_txs
    };
    txs.iter()
        .enumerate()
        .map(|(i, tx)| TxInfo::new(tx, &tx_infos[i], start_nonce + i as u64, int_latest_block_number))
        .collect::<Result<Vec<TxInfo>>>()
}

pub fn get_eos_output<D: DatabaseInterface>(state: EosState<D>) -> Result<String> {
    info!("✔ Getting EOS output json...");
    let int_signed_txs = state.eth_signed_txs;
    let output = serde_json::to_string(&EosOutput {
        #[cfg(feature = "spring_1-0")]
        eos_latest_block_number: state.eos_db_utils.get_latest_eos_block_number()?,
        #[cfg(not(feature = "spring_1-0"))]
        eos_latest_block_number: state.incremerkles.latest_block_num(),
        int_signed_transactions: if int_signed_txs.is_empty() {
            vec![]
        } else {
            let eth_db_utils = EthDbUtils::new(state.db);
            let txs = EthTransactions::from_bytes(&int_signed_txs)?;
            get_tx_infos_from_signed_txs(
                &txs,
                &IntOnEosIntTxInfos::from_bytes(&state.tx_infos)?,
                eth_db_utils.get_eth_account_nonce_from_db()?,
                eth_db_utils.get_latest_eth_block_number()?,
            )?
        },
    })?;
    info!("✔ EOS output: {}", output);
    Ok(output)
}

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
use common::errors::AppError;
#[cfg(test)]
impl FromStr for EosOutput {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
impl FromStr for TxInfo {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}
