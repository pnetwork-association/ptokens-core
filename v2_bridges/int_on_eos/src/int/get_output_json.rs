use std::time::{SystemTime, UNIX_EPOCH};

use common::{
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_eos::{EosDbUtils, EosSignedTransaction, EosSignedTransactions, Incremerkles};
use common_eth::{EthDbUtilsExt, EthState};

use crate::int::eos_tx_info::{IntOnEosEosTxInfo, IntOnEosEosTxInfos};

make_output_structs!(Int, Eos);

make_struct_with_test_assertions_on_equality_check!(
    struct EosTxInfo {
        _id: String,
        broadcast: bool,
        eos_tx_amount: String,
        int_tx_amount: String,
        eos_account_nonce: u64,
        eos_tx_recipient: String,
        eos_tx_signature: String,
        witnessed_timestamp: u64,
        eos_serialized_tx: String,
        host_token_address: String,
        originating_tx_hash: String,
        originating_address: String,
        eos_latest_block_number: u64,
        destination_chain_id: String,
        native_token_address: String,
        broadcast_tx_hash: Option<String>,
        broadcast_timestamp: Option<String>,
    }
);

impl EosTxInfo {
    pub fn new(
        eos_tx: &EosSignedTransaction,
        tx_info: &IntOnEosEosTxInfo,
        eos_account_nonce: u64,
        eos_latest_block_number: u64,
    ) -> Result<EosTxInfo> {
        Ok(EosTxInfo {
            broadcast: false,
            eos_account_nonce,
            eos_latest_block_number,
            broadcast_tx_hash: None,
            broadcast_timestamp: None,
            eos_tx_amount: eos_tx.amount.to_string(),
            eos_tx_signature: eos_tx.signature.clone(),
            eos_tx_recipient: eos_tx.recipient.clone(),
            eos_serialized_tx: eos_tx.transaction.clone(),
            int_tx_amount: tx_info.token_amount.to_string(),
            _id: format!("pint-on-eos-eos-{}", eos_account_nonce),
            host_token_address: tx_info.eos_token_address.to_string(),
            destination_chain_id: tx_info.destination_chain_id.to_hex()?,
            originating_address: format!("0x{}", hex::encode(tx_info.token_sender)),
            witnessed_timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            originating_tx_hash: format!("0x{}", hex::encode(tx_info.originating_tx_hash)),
            native_token_address: format!("0x{}", hex::encode(tx_info.eth_token_address)),
        })
    }
}

pub fn get_output_json<D: DatabaseInterface>(state: EthState<D>) -> Result<IntOutput> {
    info!("✔ Getting `IntOnEos` EOS output json...");
    Ok(IntOutput {
        int_latest_block_number: state
            .eth_db_utils
            .get_eth_latest_block_from_db()?
            .get_block_number()?
            .as_usize(),
        eos_signed_transactions: if state.signed_txs.is_empty() {
            vec![]
        } else {
            let eos_db_utils = EosDbUtils::new(state.db);
            let eos_txs = EosSignedTransactions::from_bytes(&state.signed_txs)?;
            let num_txs = eos_txs.len() as u64;
            let tx_infos = IntOnEosEosTxInfos::from_bytes(&state.tx_infos)?;

            let eos_latest_block_num = if cfg!(feature = "spring_1-0") {
                eos_db_utils.get_latest_eos_block_number()?
            } else {
                Incremerkles::get_from_db(&eos_db_utils)?.latest_block_num()
            };
            let eos_account_nonce = eos_db_utils.get_eos_account_nonce_from_db()?;
            let start_nonce = if num_txs > eos_account_nonce {
                return Err("EOS account nonce has not been incremented correctly!".into());
            } else {
                eos_account_nonce - num_txs
            };
            if tx_infos.len() as u64 != num_txs {
                return Err("Number of EOS txs does not match number of tx infos!".into());
            };
            eos_txs
                .iter()
                .enumerate()
                .map(|(i, eos_tx)| EosTxInfo::new(eos_tx, &tx_infos[i], start_nonce + i as u64, eos_latest_block_num))
                .collect::<Result<Vec<EosTxInfo>>>()?
        },
    })
}
