use std::result::Result;

use common_eth::{EthSubmissionMaterial, EthSubmissionMaterials};
use lib::{BroadcastMessages, SentinelError};

fn process_native_material_single(material: &EthSubmissionMaterial) -> Result<(), SentinelError> {
    // TODO Real pipeline
    let n = material.get_block_number()?;
    info!("Finished processing native block {n}!");
    Ok(())
}

fn process_native_material(batch: &EthSubmissionMaterials) -> Result<Vec<()>, SentinelError> {
    info!("Processing native submission material...");
    let r = batch
        .iter()
        .map(process_native_material_single)
        .collect::<Result<Vec<()>, SentinelError>>();
    info!("Finished processing native submission material!");
    r
}

// TODO need a oneshot channel for a synce to throw stuff to this thread.
// Which otherwise will do nothing until messages are received.
// all the native side needs to do is parse events that we're looking for and _save_ them. That's
// basically it! Need to save them in some performant DB, along with a "seen on opposite chain"
// type flag too.
//
// also need some way to initialize the chain since we'll need some chain concept in order to have
// the concept of confirmations
//
// also need to figure out how we're going to manage the database stuff - use something in memory
// that we can still use with references, then some sort of channel stuff to pass messages in
// between.
//
// NEED to figure out the db stuff pretty quickly to be honest, because that's the hard bit I'd
// say.
//
// also need a broadcaster, but that should be a simple module right? Which can just stay in a
// quiet loop, watching a db for txs that it might have to sign.
/*
pipeline from int side of int-on-evm:

fn submit_int_block<D: DatabaseInterface>(db: &D, json: &EthSubmissionMaterialJson) -> Result<IntOutput> {
    parse_eth_submission_material_json_and_put_in_state(json, EthState::init(db))
        .and_then(validate_eth_block_in_state)
        .and_then(|state| state.get_eth_evm_token_dictionary_and_add_to_state())
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_evm_tx_infos_from_state)
        .and_then(filter_tx_info_with_no_erc20_transfer_event)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_sign_evm_txs_and_add_to_eth_state)
        .and_then(maybe_increment_evm_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(get_int_output_json)
}
 */
