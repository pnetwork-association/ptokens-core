use crate::{
    types::Result,
    traits::DatabaseInterface,
    erc20_on_eos::{
        check_core_is_initialized::check_core_is_initialized_and_return_eos_state,
        eos::{
            get_eos_output::get_eos_output,
            sign_normal_eth_txs::maybe_sign_normal_eth_txs_and_add_to_state,
            increment_eth_nonce::maybe_increment_eth_nonce_in_db_and_return_state,
            redeem_info::{
                maybe_parse_redeem_infos_and_put_in_state,
                maybe_filter_out_already_processed_tx_ids_from_state,
            },
        },
    },
    chains::eos::{
        eos_state::EosState,
        save_latest_block_id::save_latest_block_id_to_db,
        save_latest_block_num::save_latest_block_num_to_db,
        save_incremerkle::save_incremerkle_from_state_to_db,
        validate_signature::validate_block_header_signature,
        get_eos_incremerkle::get_incremerkle_and_add_to_state,
        get_processed_tx_ids::get_processed_tx_ids_and_add_to_state,
        add_schedule::maybe_add_new_eos_schedule_to_db_and_return_state,
        validate_producer_slot::validate_producer_slot_of_block_in_state,
        get_active_schedule::get_active_schedule_from_db_and_add_to_state,
        parse_submission_material::parse_submission_material_and_add_to_state,
        eos_erc20_dictionary::get_erc20_dictionary_from_db_and_add_to_eos_state,
        append_interim_block_ids::append_interim_block_ids_to_incremerkle_in_state,
        get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
        add_global_sequences_to_processed_list::maybe_add_global_sequences_to_processed_list_and_return_state,
        eos_database_transactions::{
            end_eos_db_transaction_and_return_state,
            start_eos_db_transaction_and_return_state,
        },
        filter_action_proofs::{
            maybe_filter_duplicate_proofs_from_state,
            maybe_filter_out_proofs_for_non_erc20_accounts,
            maybe_filter_out_invalid_action_receipt_digests,
            maybe_filter_out_proofs_with_wrong_action_mroot,
            maybe_filter_out_proofs_with_invalid_merkle_proofs,
            maybe_filter_out_action_proof_receipt_mismatches_and_return_state,
        },
    },
};

/// # Submit EOS Block to Core
///
/// The main submission pipeline. Submitting an EOS block to the enclave will - if the block is
/// valid & the accompanying transaction IDs update the incremerkle correctly - advanced the core's
/// incremerkle accordingly. Any proofs submitted with the block and transaction IDs will then be
/// parsed and if found to pertain to peg outs made in the block in question, an ETH transaction
/// will be signed and returned to the caller.
pub fn submit_eos_block_to_core<D>(db: D, block_json: &str) -> Result<String> where D: DatabaseInterface {
    info!("✔ Submitting EOS block to core...");
    parse_submission_material_and_add_to_state(block_json, EosState::init(db))
        .and_then(check_core_is_initialized_and_return_eos_state)
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(get_incremerkle_and_add_to_state)
        .and_then(append_interim_block_ids_to_incremerkle_in_state)
        .and_then(get_active_schedule_from_db_and_add_to_state)
        .and_then(validate_producer_slot_of_block_in_state)
        .and_then(validate_block_header_signature)
        .and_then(start_eos_db_transaction_and_return_state)
        .and_then(get_erc20_dictionary_from_db_and_add_to_eos_state)
        .and_then(maybe_add_new_eos_schedule_to_db_and_return_state)
        .and_then(get_processed_tx_ids_and_add_to_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_proofs_for_non_erc20_accounts)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches_and_return_state)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(maybe_parse_redeem_infos_and_put_in_state)
        .and_then(maybe_filter_out_already_processed_tx_ids_from_state)
        .and_then(maybe_add_global_sequences_to_processed_list_and_return_state)
        .and_then(maybe_sign_normal_eth_txs_and_add_to_state)
        .and_then(maybe_increment_eth_nonce_in_db_and_return_state)
        .and_then(save_latest_block_id_to_db)
        .and_then(save_latest_block_num_to_db)
        .and_then(save_incremerkle_from_state_to_db)
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(get_eos_output)
}
