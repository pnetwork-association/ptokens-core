use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        eos::{
            eos_crypto::eos_transaction::{EosSignedTransaction, EosSignedTransactions},
            eos_database_utils::{get_eos_account_nonce_from_db, get_latest_eos_block_number},
        },
        eth::eth_state::EthState,
    },
    eos_on_eth::eth::eth_tx_info::EosOnEthEthTxInfo,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EosOnEthEthOutputDetails {
    pub _id: String,
    pub broadcast: bool,
    pub eos_tx_amount: String,
    pub eth_tx_amount: String,
    pub eos_account_nonce: u64,
    pub eos_tx_recipient: String,
    pub eos_tx_signature: String,
    pub witnessed_timestamp: u64,
    pub eos_serialized_tx: String,
    pub host_token_address: String,
    pub originating_tx_hash: String,
    pub originating_address: String,
    pub eos_latest_block_number: u64,
    pub native_token_address: String,
    pub broadcast_tx_hash: Option<String>,
    pub broadcast_timestamp: Option<String>,
}

impl EosOnEthEthOutputDetails {
    pub fn new(
        eos_tx: &EosSignedTransaction,
        tx_info: &EosOnEthEthTxInfo,
        eos_account_nonce: u64,
        eos_latest_block_number: u64,
    ) -> Result<EosOnEthEthOutputDetails> {
        Ok(EosOnEthEthOutputDetails {
            broadcast: false,
            eos_account_nonce,
            eos_latest_block_number,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            eos_tx_signature: eos_tx.signature.clone(),
            eos_tx_recipient: eos_tx.recipient.clone(),
            eos_serialized_tx: eos_tx.transaction.clone(),
            eth_tx_amount: tx_info.token_amount.to_string(),
            eos_tx_amount: tx_info.eos_asset_amount.clone(),
            _id: format!("peos-on-eth-eos-{}", eos_account_nonce),
            host_token_address: format!("0x{}", hex::encode(&tx_info.eth_token_address)),
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender)),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            originating_tx_hash: format!("0x{}", hex::encode(&tx_info.originating_tx_hash)),
            native_token_address: tx_info.eos_token_address.to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EosOnEthEthOutput {
    pub eth_latest_block_number: u64,
    pub eos_signed_transactions: Vec<EosOnEthEthOutputDetails>,
}

fn check_eos_nonce_is_sufficient<D: DatabaseInterface>(db: &D, eos_txs: &EosSignedTransactions) -> Result<u64> {
    get_eos_account_nonce_from_db(db).and_then(|eos_nonce| {
        if eos_nonce >= eos_txs.len() as u64 {
            Ok(eos_nonce)
        } else {
            Err("EOS nonce is NOT greater than or equal to the number of EOS txs!".into())
        }
    })
}

pub fn get_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
    info!("✔ Getting `eos-on-eth` ETH submission output json...");
    Ok(serde_json::to_string(&EosOnEthEthOutput {
        eth_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_u64(),
        eos_signed_transactions: match state.eos_transactions {
            None => vec![],
            Some(ref eos_txs) => {
                let eos_nonce = check_eos_nonce_is_sufficient(state.db, eos_txs)?;
                let start_nonce = eos_nonce - eos_txs.len() as u64;
                eos_txs
                    .iter()
                    .enumerate()
                    .map(|(i, eos_tx)| {
                        EosOnEthEthOutputDetails::new(
                            eos_tx,
                            &state.eos_on_eth_eth_tx_infos[i],
                            start_nonce + i as u64,
                            get_latest_eos_block_number(state.db)?,
                        )
                    })
                    .collect::<Result<Vec<EosOnEthEthOutputDetails>>>()?
            },
        },
    })?)
}
