use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use bitcoin::blockdata::transaction::Transaction as BtcTransaction;
use derive_more::Constructor;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{
    btc_on_int::int::btc_tx_info::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
    chains::{
        btc::btc_utils::get_hex_tx_from_signed_btc_tx,
        eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    },
    errors::AppError,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Constructor)]
pub struct IntOutput {
    pub int_latest_block_number: usize,
    pub btc_signed_transactions: Vec<BtcTxInfo>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct BtcTxInfo {
    pub _id: String,
    pub broadcast: bool,
    pub btc_tx_hash: String,
    pub btc_tx_amount: u64,
    pub btc_signed_tx: String,
    pub btc_account_nonce: u64,
    pub witnessed_timestamp: u64,
    pub host_token_address: String,
    pub originating_address: String,
    pub originating_tx_hash: String,
    pub destination_address: String,
    pub btc_latest_block_number: u64,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<usize>,
}

#[cfg(test)]
impl FromStr for BtcTxInfo {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

#[cfg(test)]
impl FromStr for IntOutput {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        #[derive(Serialize, Deserialize)]
        struct Interim {
            int_latest_block_number: usize,
            btc_signed_transactions: Vec<JsonValue>,
        }
        let interim = serde_json::from_str::<Interim>(s)?;
        let tx_infos = interim
            .btc_signed_transactions
            .iter()
            .map(|json| BtcTxInfo::from_str(&json.to_string()))
            .collect::<Result<Vec<BtcTxInfo>>>()?;
        Ok(Self {
            int_latest_block_number: interim.int_latest_block_number,
            btc_signed_transactions: tx_infos,
        })
    }
}

impl BtcTxInfo {
    pub fn new(
        btc_tx: &BtcTransaction,
        tx_info: &BtcOnIntBtcTxInfo,
        btc_account_nonce: u64,
        btc_latest_block_number: u64,
        host_token_address: &EthAddress,
    ) -> Result<BtcTxInfo> {
        Ok(BtcTxInfo {
            broadcast: false,
            btc_account_nonce,
            broadcast_tx_hash: None,
            btc_latest_block_number,
            broadcast_timestamp: None,
            btc_tx_hash: btc_tx.txid().to_string(),
            btc_tx_amount: tx_info.amount_in_satoshis,
            destination_address: tx_info.recipient.clone(),
            _id: format!("pbtc-on-int-btc-{btc_account_nonce}"),
            btc_signed_tx: get_hex_tx_from_signed_btc_tx(btc_tx),
            host_token_address: format!("0x{}", hex::encode(host_token_address)),
            originating_address: format!("0x{}", hex::encode(tx_info.from.as_bytes())),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash.as_bytes())),
        })
    }
}

pub fn get_btc_signed_tx_info_from_btc_txs(
    btc_account_nonce: u64,
    btc_txs: Vec<BtcTransaction>,
    redeem_infos: &BtcOnIntBtcTxInfos,
    btc_latest_block_number: u64,
    host_token_address: &EthAddress,
) -> Result<Vec<BtcTxInfo>> {
    info!("✔ Getting BTC tx info from {} BTC tx(s)...", btc_txs.len());
    let num_btc_txs = btc_txs.len();
    let num_redeem_infos = redeem_infos.len();
    if num_btc_txs > num_redeem_infos {
        // NOTE: There CAN be fewer such as in the case of txs being filtered out for amounts being too low.
        return Err(format!(
            "There are MORE txs than tx infos! Num BTC txs: {}, Num RedeemInfos: {}",
            num_btc_txs, num_redeem_infos
        )
        .into());
    };
    let start_nonce = btc_account_nonce - btc_txs.len() as u64;
    btc_txs
        .iter()
        .enumerate()
        .map(|(i, btc_tx)| {
            BtcTxInfo::new(
                btc_tx,
                &redeem_infos.0[i],
                start_nonce + i as u64,
                btc_latest_block_number,
                host_token_address,
            )
        })
        .collect::<Result<Vec<_>>>()
}

pub fn get_int_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting INT output json...");
    let output = serde_json::to_string(&IntOutput {
        int_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_usize(),
        btc_signed_transactions: match state.btc_transactions {
            Some(txs) => get_btc_signed_tx_info_from_btc_txs(
                state.btc_db_utils.get_btc_account_nonce_from_db()?,
                txs,
                &state.btc_on_int_btc_tx_infos,
                state.btc_db_utils.get_latest_btc_block_number()?,
                &state.eth_db_utils.get_btc_on_int_smart_contract_address_from_db()?,
            )?,
            None => vec![],
        },
    })?;
    info!("✔ INT Output: {}", output);
    Ok(output)
}
