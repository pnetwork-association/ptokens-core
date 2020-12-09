use crate::{
    btc_on_eos::{
        btc::{
            get_btc_output_json::{create_btc_output_json_and_put_in_state, get_btc_output_as_string},
            minting_params::parse_minting_params_from_p2sh_deposits_and_add_to_state,
            sign_transactions::maybe_sign_canon_block_txs_and_add_to_state,
        },
        check_core_is_initialized::check_core_is_initialized_and_return_btc_state,
    },
    chains::btc::{
        add_btc_block_to_db::maybe_add_btc_block_to_db,
        btc_database_utils::{end_btc_db_transaction, start_btc_db_transaction},
        btc_state::BtcState,
        btc_submission_material::parse_submission_material_and_put_in_state,
        check_btc_parent_exists::check_for_parent_of_btc_block_in_state,
        extract_utxos_from_p2sh_txs::maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
        filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
        filter_utxos::filter_out_value_too_low_utxos_from_state,
        get_btc_block_in_db_format::create_btc_block_in_db_format_and_put_in_state,
        get_deposit_info_hash_map::get_deposit_info_hash_map_and_put_in_state,
        increment_eos_nonce::maybe_increment_eos_nonce,
        remove_minting_params_from_canon_block::remove_minting_params_from_canon_block_and_return_state,
        remove_old_btc_tail_block::maybe_remove_old_btc_tail_block,
        save_utxos_to_db::maybe_save_utxos_to_db,
        update_btc_canon_block_hash::maybe_update_btc_canon_block_hash,
        update_btc_latest_block_hash::maybe_update_btc_latest_block_hash,
        update_btc_linker_hash::maybe_update_btc_linker_hash,
        update_btc_tail_block_hash::maybe_update_btc_tail_block_hash,
        validate_btc_block_header::validate_btc_block_header_in_state,
        validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
        validate_btc_merkle_root::validate_btc_merkle_root,
        deposit_address_info::validate_deposit_address_list_in_state,
        validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn submit_btc_block_to_core<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting BTC block to core...");
    parse_submission_material_and_put_in_state(block_json_string, BtcState::init(db))
        .and_then(check_core_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(check_for_parent_of_btc_block_in_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(validate_deposit_address_list_in_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_minting_params_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(filter_out_value_too_low_utxos_from_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(create_btc_block_in_db_format_and_put_in_state)
        .and_then(maybe_add_btc_block_to_db)
        .and_then(maybe_update_btc_latest_block_hash)
        .and_then(maybe_update_btc_canon_block_hash)
        .and_then(maybe_update_btc_tail_block_hash)
        .and_then(maybe_update_btc_linker_hash)
        .and_then(maybe_sign_canon_block_txs_and_add_to_state)
        .and_then(maybe_increment_eos_nonce)
        .and_then(maybe_remove_old_btc_tail_block)
        .and_then(create_btc_output_json_and_put_in_state)
        .and_then(remove_minting_params_from_canon_block_and_return_state)
        .and_then(end_btc_db_transaction)
        .and_then(get_btc_output_as_string)
}
