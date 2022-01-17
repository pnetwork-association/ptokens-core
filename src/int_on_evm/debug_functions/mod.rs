pub(crate) mod block_reprocessors;

use ethereum_types::U256;
use serde_json::json;

use crate::{
    chains::eth::{
        eth_constants::{EthDatabaseKeysJson, ETH_PRIVATE_KEY_DB_KEY},
        eth_contracts::erc20_vault::{
            encode_erc20_vault_add_supported_token_fx_data,
            encode_erc20_vault_peg_out_fxn_data_without_user_data,
            encode_erc20_vault_remove_supported_token_fx_data,
        },
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
        eth_debug_functions::{debug_set_eth_gas_price_in_db, debug_set_evm_gas_price_in_db},
        eth_utils::convert_hex_to_eth_address,
        evm_constants::{EvmDatabaseKeysJson, EVM_PRIVATE_KEY_DB_KEY},
    },
    check_debug_mode::check_debug_mode,
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL},
    debug_database_utils::{get_key_from_db, set_key_in_db_to_value},
    dictionaries::{
        dictionary_constants::ETH_EVM_DICTIONARY_KEY,
        eth_evm::{EthEvmTokenDictionary, EthEvmTokenDictionaryEntry},
    },
    fees::fee_utils::sanity_check_basis_points_value,
    int_on_evm::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Get All DB Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().map(|_| {
        json!({
            "evm": EvmDatabaseKeysJson::new(),
            "eth": EthDatabaseKeysJson::new(),
            "db-key-prefix": DB_KEY_PREFIX.to_string(),
            "dictionary": hex::encode(ETH_EVM_DICTIONARY_KEY.to_vec()),
        })
        .to_string()
    })
}

/// # Debug Set Key in DB to Value
///
/// This function set to the given value a given key in the encryped database.
///
/// ### BEWARE:
/// Only use this if you know exactly what you are doing and why.
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(db: D, key: &str, value: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| {
            let key_bytes = hex::decode(&key)?;
            let sensitivity =
                match key_bytes == ETH_PRIVATE_KEY_DB_KEY.to_vec() || key_bytes == EVM_PRIVATE_KEY_DB_KEY.to_vec() {
                    true => MAX_DATA_SENSITIVITY_LEVEL,
                    false => None,
                };
            set_key_in_db_to_value(db, key, value, sensitivity)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Get Key From Db
///
/// This function will return the value stored under a given key in the encrypted database.
pub fn debug_get_key_from_db<D: DatabaseInterface>(db: D, key: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| {
            let key_bytes = hex::decode(&key)?;
            let sensitivity =
                match key_bytes == ETH_PRIVATE_KEY_DB_KEY.to_vec() || key_bytes == EVM_PRIVATE_KEY_DB_KEY.to_vec() {
                    true => MAX_DATA_SENSITIVITY_LEVEL,
                    false => None,
                };
            get_key_from_db(db, key, sensitivity)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Add Dictionary Entry
///
/// This function will add an entry to the `EthEvmTokenDictionary` held in the encrypted database. The
/// dictionary defines the relationship between ETH token addresses and the address of their pTokenized,
/// EVM-compliant counterparts.
///
/// The required format of an entry is:
/// {
///     "eth_symbol": <symbol>,
///     "evm_symbol": <symbol>,
///     "eth_address": <address>,
///     "evm_address": <address>,
/// }
pub fn debug_add_dictionary_entry<D: DatabaseInterface>(db: D, json_str: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(&db), &EvmDbUtils::new(&db)))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthEvmTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| dictionary.add_and_update_in_db(EthEvmTokenDictionaryEntry::from_str(json_str)?, &db))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"add_dictionary_entry_success:":"true"}).to_string())
}

/// # Debug Remove Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in ETH address from the
/// `EthEvmTokenDictionaryEntry` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
pub fn debug_remove_dictionary_entry<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(&db), &EvmDbUtils::new(&db)))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthEvmTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| {
            dictionary.remove_entry_via_eth_address_and_update_in_db(&convert_hex_to_eth_address(eth_address_str)?, &db)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"remove_dictionary_entry_success:":"true"}).to_string())
}

/// # Debug Get Add Supported Token Transaction
///
/// This function will sign a transaction to add the given address as a supported token to
/// the `erc20-vault-on-eos` smart-contract.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function will increment the core's ETH nonce, and so if the transaction is not broadcast
/// successfully, the core's ETH side will no longer function correctly. Use with extreme caution
/// and only if you know exactly what you are doing and why!
pub fn debug_get_add_supported_token_tx<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    info!("✔ Debug getting `addSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(&db);
    let evm_db_utils = EvmDbUtils::new(&db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = convert_hex_to_eth_address(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &evm_db_utils))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_add_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                &chain_id,
                chain_id.get_erc20_vault_change_supported_token_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({ "success": true, "eth_signed_tx": hex_tx }).to_string())
        })
}

/// # Debug Get Remove Supported Token Transaction
///
/// This function will sign a transaction to remove the given address as a supported token to
/// the `erc20-vault-on-eos` smart-contract.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function will increment the core's ETH nonce, and so if the transaction is not broadcast
/// successfully, the core's ETH side will no longer function correctly. Use with extreme caution
/// and only if you know exactly what you are doing and why!
pub fn debug_get_remove_supported_token_tx<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    info!("✔ Debug getting `removeSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(&db);
    let evm_db_utils = EvmDbUtils::new(&db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = convert_hex_to_eth_address(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &evm_db_utils))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_remove_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                &chain_id,
                chain_id.get_erc20_vault_change_supported_token_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({ "success": true, "eth_signed_tx": hex_tx }).to_string())
        })
}

/// # Debug Set Fee Basis Points
///
/// This function takes an address and a new fee param. It gets the `EthEvmTokenDictionary` from
/// the database then finds the entry pertaining to the address in question and if successful,
/// updates the fee associated with that address before saving the dictionary back into the
/// database. If no entry is found for a given `address` the function will return an error saying
/// as such.
///
/// #### NOTE: Using a fee of 0 will mean no fees are taken.
pub fn debug_set_fee_basis_points<D: DatabaseInterface>(db: D, address: &str, new_fee: u64) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(&db), &EvmDbUtils::new(&db)))
        .map(|_| sanity_check_basis_points_value(new_fee))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthEvmTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| {
            dictionary.change_fee_basis_points_and_update_in_db(&db, &convert_hex_to_eth_address(address)?, new_fee)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true, "address": address, "new_fee": new_fee}).to_string())
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Withdraw Fees
///
/// This function takes an address and uses it to search through the token dictionary to find a
/// corresponding entry. Once found, that entry's accrued fees are zeroed, a timestamp set in that
/// entry to mark the withdrawal date and the dictionary saved back in the database. Finally, an
/// ETH transaction is created to transfer the `<accrued_fees>` amount of tokens to the passed in
/// recipient address.
///
/// #### NOTE: This function will increment the ETH nonce and so the output transation MUST be
/// broadcast otherwise future transactions are liable to fail.
pub fn debug_withdraw_fees_and_save_in_db<D: DatabaseInterface>(
    db: D,
    token_address: &str,
    recipient_address: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(&db);
    let evm_db_utils = EvmDbUtils::new(&db);
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &evm_db_utils))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthEvmTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| {
            dictionary.withdraw_fees_and_save_in_db(&db, &convert_hex_to_eth_address(token_address)?)
        })
        .and_then(|(token_address, fee_amount)| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                encode_erc20_vault_peg_out_fxn_data_without_user_data(
                    convert_hex_to_eth_address(recipient_address)?,
                    token_address,
                    fee_amount,
                )?,
                eth_db_utils.get_eth_account_nonce_from_db()?,
                0,
                eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                &chain_id,
                chain_id.get_erc20_vault_pegout_without_user_data_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            eth_db_utils.increment_eth_account_nonce_in_db(1)?;
            db.end_transaction()?;
            Ok(json!({"success": true, "eth_signed_tx": hex_tx}).to_string())
        })
}

/// # Debug Set EVM Gas Price
///
/// This function sets the EVM gas price to use when making EVM transactions. It's unit is `Wei`.
pub fn debug_set_evm_gas_price<D: DatabaseInterface>(db: D, gas_price: u64) -> Result<String> {
    // NOTE: This alias exists so as not to break the legacy API where the DB is NOT a reference!
    debug_set_evm_gas_price_in_db(&db, gas_price)
}

/// # Debug Set INT Gas Price
///
/// This function sets the INT gas price to use when making ETH transactions. It's unit is `Wei`.
pub fn debug_set_int_gas_price<D: DatabaseInterface>(db: D, gas_price: u64) -> Result<String> {
    // NOTE: This alias exists so as not to break the legacy API where the DB is NOT a reference!
    debug_set_eth_gas_price_in_db(&db, gas_price)
}

/// # Debug Set Accrued Fees
///
/// This function updates the accrued fees value in the dictionary entry retrieved from the passed
/// in ETH address.
pub fn debug_set_accrued_fees_in_dictionary<D: DatabaseInterface>(
    db: D,
    token_address: &str,
    fee_amount: String,
) -> Result<String> {
    info!("✔ Debug setting accrued fees in dictionary...");
    let dictionary = EthEvmTokenDictionary::get_from_db(&db)?;
    let dictionary_entry_eth_address = convert_hex_to_eth_address(token_address)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(&db), &EvmDbUtils::new(&db)))
        .and_then(|_| db.start_transaction())
        .and_then(|_| {
            dictionary.set_accrued_fees_and_save_in_db(
                &db,
                &dictionary_entry_eth_address,
                U256::from_dec_str(&fee_amount)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true,"fee":fee_amount}).to_string())
}
