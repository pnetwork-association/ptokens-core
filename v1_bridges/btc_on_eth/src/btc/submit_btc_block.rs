use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_btc::{
    check_for_parent_of_btc_block_in_state,
    create_btc_block_in_db_format_and_put_in_state,
    end_btc_db_transaction,
    filter_out_value_too_low_utxos_from_state,
    filter_p2sh_deposit_txs_and_add_to_state,
    get_deposit_info_hash_map_and_put_in_state,
    maybe_add_btc_block_to_db,
    maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
    maybe_remove_old_btc_tail_block,
    maybe_save_utxos_to_db,
    maybe_update_btc_canon_block_hash,
    maybe_update_btc_latest_block_hash,
    maybe_update_btc_linker_hash,
    maybe_update_btc_tail_block_hash,
    parse_btc_block_and_id_and_put_in_state,
    parse_btc_submission_json_and_put_in_state,
    remove_tx_infos_from_canon_block_and_return_state,
    set_any_sender_flag_in_state,
    validate_btc_block_header_in_state,
    validate_btc_merkle_root,
    validate_deposit_address_list_in_state,
    validate_difficulty_of_btc_block_in_state,
    validate_proof_of_work_of_btc_block_in_state,
    BtcState,
};

use crate::btc::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
    filter_eth_tx_infos::maybe_filter_out_value_too_low_btc_on_eth_eth_tx_infos_in_state,
    get_btc_output_json::{create_btc_output_json_and_put_in_state, get_btc_output_as_string},
    maybe_increment_nonce_in_db,
    parse_tx_infos::parse_eth_tx_infos_from_p2sh_deposits_and_add_to_state,
    sign_any_sender_transactions::maybe_sign_any_sender_canon_block_txs_and_add_to_state,
    sign_normal_eth_transactions::maybe_sign_normal_canon_block_txs_and_add_to_state,
};

/// # Submit BTC Block to Enclave
///
/// The main submission pipeline. Submitting a BTC block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the BTC
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a deposit to an address derived from the enclave's BTC public key, an ETH
/// transaction will be signed & returned to the caller.
pub fn submit_btc_block_to_enclave<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting BTC block to enclave...");
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| parse_btc_submission_json_and_put_in_state(block_json_string, BtcState::init(db)))
        .and_then(set_any_sender_flag_in_state)
        .and_then(parse_btc_block_and_id_and_put_in_state)
        .and_then(check_for_parent_of_btc_block_in_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(validate_deposit_address_list_in_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_eth_tx_infos_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(filter_out_value_too_low_utxos_from_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(maybe_filter_out_value_too_low_btc_on_eth_eth_tx_infos_in_state)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(create_btc_block_in_db_format_and_put_in_state)
        .and_then(maybe_add_btc_block_to_db)
        .and_then(maybe_update_btc_latest_block_hash)
        .and_then(maybe_update_btc_canon_block_hash)
        .and_then(maybe_update_btc_tail_block_hash)
        .and_then(maybe_update_btc_linker_hash)
        .and_then(maybe_sign_normal_canon_block_txs_and_add_to_state)
        .and_then(maybe_sign_any_sender_canon_block_txs_and_add_to_state)
        .and_then(maybe_increment_nonce_in_db)
        .and_then(maybe_remove_old_btc_tail_block)
        .and_then(create_btc_output_json_and_put_in_state)
        .and_then(remove_tx_infos_from_canon_block_and_return_state)
        .and_then(end_btc_db_transaction)
        .and_then(get_btc_output_as_string)
}
