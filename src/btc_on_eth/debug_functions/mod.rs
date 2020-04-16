use crate::{
    types::Result,
    traits::DatabaseInterface,
    check_debug_mode::check_debug_mode,
    chains::btc::utxo_manager::utxo_utils::get_all_utxos_as_json_string,
    debug_database_utils::{
        get_key_from_db,
        set_key_in_db_to_value,
    },
    btc_on_eth::{
        check_enclave_is_initialized::{
            check_enclave_is_initialized,
            check_enclave_is_initialized_and_return_eth_state,
            check_enclave_is_initialized_and_return_btc_state,
        },
        btc::{
            btc_state::BtcState,
            sign_transactions::get_eth_signed_txs,
            save_utxos_to_db::maybe_save_utxos_to_db,
            filter_utxos::maybe_filter_utxos_in_state,
            btc_constants::BTC_PRIVATE_KEY_DB_KEY as BTC_KEY,
            validate_btc_merkle_root::validate_btc_merkle_root,
            increment_eth_nonce::maybe_increment_eth_nonce_in_db,
            parse_btc_block::parse_btc_block_and_id_and_put_in_state,
            get_btc_output_json::get_eth_signed_tx_info_from_eth_txs,
            filter_minting_params::maybe_filter_minting_params_in_state,
            validate_btc_block_header::validate_btc_block_header_in_state,
            filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
            btc_database_utils::{
                end_btc_db_transaction,
                start_btc_db_transaction,
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
            extract_utxos_from_p2sh_txs::{
                maybe_extract_utxos_from_p2sh_txs_and_put_in_state
            },
            extract_utxos_from_op_return_txs::{
                maybe_extract_utxos_from_op_return_txs_and_put_in_state,
            },
            parse_minting_params_from_p2sh_deposits::{
                parse_minting_params_from_p2sh_deposits_and_add_to_state,
            },
            parse_minting_params_from_op_return_deposits::{
                parse_minting_params_from_op_return_deposits_and_add_to_state,
            },
        },
        eth::{
            eth_state::EthState,
            validate_block::validate_block_in_state,
            save_btc_utxos_to_db::maybe_save_btc_utxos_to_db,
            eth_constants::ETH_PRIVATE_KEY_DB_KEY as ETH_KEY,
            parse_redeem_params::parse_redeem_params_from_block,
            increment_btc_nonce::maybe_increment_btc_nonce_in_db,
            filter_receipts::filter_irrelevant_receipts_from_state,
            create_btc_transactions::maybe_create_btc_txs_and_add_to_state,
            eth_database_utils::{
                end_eth_db_transaction,
                start_eth_db_transaction,
                get_signing_params_from_db,
                get_eth_account_nonce_from_db,
            },
            get_eth_output_json::{
                EthOutput,
                get_btc_signed_tx_info_from_btc_txs,
            },
            extract_utxos_from_btc_txs::{
                maybe_extract_btc_utxo_from_btc_tx_in_state,
            },
            parse_eth_block_and_receipts::{
                parse_eth_block_and_receipts_and_put_in_state,
            },
        },
    },
};

pub fn debug_reprocess_btc_block<D>(
    db: D,
    btc_block_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    parse_btc_block_and_id_and_put_in_state(
        btc_block_json,
        BtcState::init(db),
    )
        .and_then(check_enclave_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(validate_btc_block_header_in_state)
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
        .and_then(|state| {
            get_eth_signed_txs(
                &get_signing_params_from_db(&state.db)?,
                &state.minting_params,
            )
                .and_then(|signed_txs| state.add_eth_signed_txs(signed_txs))
        })
        .and_then(maybe_increment_eth_nonce_in_db)
        .and_then(|state| {
            let signatures = serde_json::to_string(
                &match &state.eth_signed_txs {
                    None => Ok(vec![]),
                    Some(txs) =>
                        get_eth_signed_tx_info_from_eth_txs(
                            txs,
                            &state.minting_params,
                            get_eth_account_nonce_from_db(&state.db)?,
                        )
                }?
            )?;
            info!("✔ BTC signatures: {}", signatures);
            state.add_output_json_string(signatures)
        })
        .and_then(end_btc_db_transaction)
        .map(|state|
            match state.output_json_string {
                None => "✘ No signatures signed ∴ no output!".to_string(),
                Some(output) => output
            }
        )
}

pub fn debug_reprocess_eth_block<D>(
    db: D,
    eth_block_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    parse_eth_block_and_receipts_and_put_in_state(
        eth_block_json,
        EthState::init(db),
    )
        .and_then(check_enclave_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction)
        .and_then(validate_block_in_state)
        .and_then(filter_irrelevant_receipts_from_state)
        .and_then(|state| {
            state
                .get_eth_block_and_receipts()
                .and_then(|block| parse_redeem_params_from_block(block.clone()))
                .and_then(|params| state.add_redeem_params(params))
        })
        .and_then(maybe_create_btc_txs_and_add_to_state)
        .and_then(maybe_increment_btc_nonce_in_db)
        .and_then(maybe_extract_btc_utxo_from_btc_tx_in_state)
        .and_then(maybe_save_btc_utxos_to_db)
        .and_then(end_eth_db_transaction)
        .and_then(|state| {
            info!("✔ Getting ETH output json...");
            let output = serde_json::to_string(
                &EthOutput {
                    eth_latest_block_number: 0,
                    btc_signed_transactions: match state.btc_transactions {
                        Some(txs) => get_btc_signed_tx_info_from_btc_txs(
                            0,
                            txs,
                            &state.redeem_params,
                        )?,
                        None => vec![],
                    }
                }
            )?;
            info!("✔ ETH Output: {}", output);
            Ok(output)
        })
}

pub fn debug_set_key_in_db_to_value<D>(
    db: D,
    key: String,
    value: String
) -> Result<String>
    where D: DatabaseInterface
{
    check_enclave_is_initialized(&db)
        .and_then(|_| set_key_in_db_to_value(db, key, value))
}

pub fn debug_get_key_from_db<D>(
    db: D,
    key: String
) -> Result<String>
    where D: DatabaseInterface
{
    let key_bytes = hex::decode(&key)?;
    check_enclave_is_initialized(&db)
        .and_then(|_| {
            if key_bytes == ETH_KEY || key_bytes == BTC_KEY {
                get_key_from_db(db, key, Some(255))
            } else {
                get_key_from_db(db, key, None)
            }
        })
}

pub fn debug_get_all_utxos<D>(
    db: D
) -> Result<String>
    where D: DatabaseInterface
{
    check_debug_mode()
        .and_then(|_| check_enclave_is_initialized(&db))
        .and_then(|_| get_all_utxos_as_json_string(db))
}
