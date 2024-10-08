use common::{
    core_type::CoreType,
    traits::{DatabaseInterface, Serdable},
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_btc::{BtcDbUtils, BtcTransactions};
use common_debug_signers::validate_debug_command_signature;
use common_eth::{
    end_eth_db_transaction_and_return_state,
    parse_eth_submission_material_and_put_in_state,
    validate_eth_block_in_state,
    EthDbUtilsExt,
    EthState,
};
use function_name::named;

use crate::{
    constants::CORE_TYPE,
    int::{
        debug_filter_tx_info_with_no_erc20_transfer_event,
        filter_receipts_for_btc_on_int_redeem_events_in_state,
        get_btc_signed_tx_info_from_btc_txs,
        maybe_increment_btc_account_nonce_and_return_eth_state,
        maybe_sign_btc_txs_and_add_to_state,
        BtcOnIntBtcTxInfos,
        IntOutput,
    },
};

#[named]
fn reprocess_int_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json, EthState::init(db)))
        .and_then(validate_eth_block_in_state)
        .and_then(filter_receipts_for_btc_on_int_redeem_events_in_state)
        .and_then(|state| {
            state
                .get_eth_submission_material()
                .and_then(|material| {
                    BtcOnIntBtcTxInfos::from_eth_submission_material(
                        material,
                        &state.eth_db_utils.get_btc_on_eth_smart_contract_address_from_db()?,
                    )
                })
                .and_then(|params| params.to_bytes())
                .map(|bytes| state.add_tx_infos(bytes))
        })
        .and_then(debug_filter_tx_info_with_no_erc20_transfer_event)
        .and_then(maybe_sign_btc_txs_and_add_to_state)
        .and_then(maybe_increment_btc_account_nonce_and_return_eth_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(|state| {
            info!("✔ Getting INT output json...");
            let output = serde_json::to_string(&IntOutput {
                int_latest_block_number: state.eth_db_utils.get_latest_eth_block_number()?,
                btc_signed_transactions: if state.signed_txs.is_empty() {
                    vec![]
                } else {
                    let txs = BtcTransactions::from_bytes(&state.signed_txs)?;
                    let btc_db_utils = BtcDbUtils::new(state.db);
                    get_btc_signed_tx_info_from_btc_txs(
                        btc_db_utils.get_btc_account_nonce_from_db()?,
                        txs,
                        &BtcOnIntBtcTxInfos::from_bytes(&state.tx_infos)?,
                        btc_db_utils.get_latest_btc_block_number()?,
                        &state.eth_db_utils.get_btc_on_int_smart_contract_address_from_db()?,
                        &btc_db_utils.get_btc_chain_id_from_db()?,
                    )?
                },
            })?;
            info!("✔ INT Output: {}", output);
            Ok(output)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess INT Block
///
/// This function will take a passed in INT block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTE:
///
///  - This function will increment the core's INT nonce, meaning the outputted reports will have a gap in their report
///    IDs!
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future BTC transactions will
/// fail due to the core having an incorret set of UTXOs!
pub fn debug_reprocess_int_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    reprocess_int_block(db, block_json, signature)
}
