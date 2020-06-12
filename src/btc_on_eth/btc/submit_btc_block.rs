use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::{
        check_core_is_initialized::{
            check_core_is_initialized_and_return_btc_state,
        },
        btc::{
            btc_state::BtcState,
            save_utxos_to_db::maybe_save_utxos_to_db,
            filter_utxos::maybe_filter_utxos_in_state,
            add_btc_block_to_db::maybe_add_btc_block_to_db,
            validate_btc_merkle_root::validate_btc_merkle_root,
            update_btc_linker_hash::maybe_update_btc_linker_hash,
            increment_eth_nonce::maybe_increment_eth_nonce_in_db,
            parse_btc_block::parse_btc_block_and_id_and_put_in_state,
            remove_old_btc_tail_block::maybe_remove_old_btc_tail_block,
            filter_minting_params::maybe_filter_minting_params_in_state,
            update_btc_tail_block_hash::maybe_update_btc_tail_block_hash,
            validate_btc_block_header::validate_btc_block_header_in_state,
            update_btc_canon_block_hash::maybe_update_btc_canon_block_hash,
            check_btc_parent_exists::check_for_parent_of_btc_block_in_state,
            update_btc_latest_block_hash::maybe_update_btc_latest_block_hash,
            filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
            validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
            btc_database_utils::{
                end_btc_db_transaction,
                start_btc_db_transaction,
            },
            get_btc_output_json::{
                get_btc_output_as_string,
                create_btc_output_json_and_put_in_state,
            },
            get_deposit_info_hash_map::{
                get_deposit_info_hash_map_and_put_in_state,
            },
            validate_btc_proof_of_work::{
                validate_proof_of_work_of_btc_block_in_state,
            },
            filter_op_return_deposit_txs::{
                filter_op_return_deposit_txs_and_add_to_state,
            },
            get_btc_block_in_db_format::{
                create_btc_block_in_db_format_and_put_in_state
            },
            extract_utxos_from_p2sh_txs::{
                maybe_extract_utxos_from_p2sh_txs_and_put_in_state
            },
            sign_transactions::{
                maybe_sign_canon_block_transactions_and_add_to_state,
            },
            extract_utxos_from_op_return_txs::{
                maybe_extract_utxos_from_op_return_txs_and_put_in_state,
            },
            remove_minting_params_from_canon_block::{
                remove_minting_params_from_canon_block_and_return_state,
            },
            parse_minting_params_from_p2sh_deposits::{
                parse_minting_params_from_p2sh_deposits_and_add_to_state,
            },
            parse_minting_params_from_op_return_deposits::{
                parse_minting_params_from_op_return_deposits_and_add_to_state,
            },
        },
    },
};

#[cfg(feature = "any-sender")]
use crate::btc_on_eth::btc::increment_any_sender_nonce::maybe_increment_any_sender_nonce_in_db;

pub fn submit_btc_block_to_enclave<D>(
    db: D,
    block_json_string: String
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Submitting BTC block to enclave...");
    parse_btc_block_and_id_and_put_in_state(
        block_json_string,
        BtcState::init(db),
    )
        .and_then(check_core_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(check_for_parent_of_btc_block_in_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(filter_op_return_deposit_txs_and_add_to_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_minting_params_from_op_return_deposits_and_add_to_state)
        .and_then(parse_minting_params_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_extract_utxos_from_op_return_txs_and_put_in_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(maybe_filter_utxos_in_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(maybe_filter_minting_params_in_state)
        .and_then(create_btc_block_in_db_format_and_put_in_state)
        .and_then(maybe_add_btc_block_to_db)
        .and_then(maybe_update_btc_latest_block_hash)
        .and_then(maybe_update_btc_canon_block_hash)
        .and_then(maybe_update_btc_tail_block_hash)
        .and_then(maybe_update_btc_linker_hash)
        .and_then(maybe_sign_canon_block_transactions_and_add_to_state)
        .and_then(maybe_increment_eth_nonce_in_db)
        .and_then(|result| {
            #[cfg(feature = "any-sender")]
            return maybe_increment_any_sender_nonce_in_db(result);

            #[cfg(not(feature = "any-sender"))]
            Ok(result)
        })
        .and_then(maybe_remove_old_btc_tail_block)
        .and_then(create_btc_output_json_and_put_in_state)
        .and_then(remove_minting_params_from_canon_block_and_return_state)
        .and_then(end_btc_db_transaction)
        .and_then(get_btc_output_as_string)
}
