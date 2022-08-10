pub(crate) mod eos_block_reprocessor;
pub(crate) mod eth_block_reprocessor;

use std::str::FromStr;

use eos_chain::{AccountName as EosAccountName, Action as EosAction, PermissionLevel, Transaction as EosTransaction};
use ethereum_types::U256;
use serde_json::json;

use crate::{
    chains::{
        eos::{
            eos_actions::PTokenPegOutAction,
            eos_constants::{EOS_ACCOUNT_PERMISSION_LEVEL, PEGOUT_ACTION_NAME},
            eos_crypto::{eos_private_key::EosPrivateKey, eos_transaction::EosSignedTransaction},
            eos_database_utils::{EosDatabaseKeysJson, EosDbUtils},
            eos_utils::get_eos_tx_expiration_timestamp_with_offset,
        },
        eth::{
            eth_database_utils::{EthDatabaseKeysJson, EthDbUtils},
            eth_utils::convert_hex_to_eth_address,
        },
    },
    constants::DB_KEY_PREFIX,
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    dictionaries::{dictionary_constants::EOS_ETH_DICTIONARY_KEY, eos_eth::EosEthTokenDictionary},
    eos_on_eth::check_core_is_initialized::check_core_is_initialized,
    fees::fee_utils::sanity_check_basis_points_value,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().and(Ok(json!({
        "eth": EthDatabaseKeysJson::new(),
        "eos": EosDatabaseKeysJson::new(),
        "db-key-prefix": DB_KEY_PREFIX.to_string(),
        "dictionary:": hex::encode(EOS_ETH_DICTIONARY_KEY.to_vec()),
    })
    .to_string()))
}

/// # Debug Set ETH Fee Basis Points
///
/// This function takes an address and a new fee param. It gets the `EosEthTokenDictionary` from
/// the database then finds the entry pertaining to the address in question and if successful,
/// updates the fee associated with that address before saving the dictionary back into the
/// database. If no entry is found for a given `address` the function will return an error saying
/// as such.
///
/// #### NOTE: Using a fee of 0 will mean no fees are taken.
pub fn debug_set_eth_fee_basis_points<D: DatabaseInterface>(
    db: &D,
    address: &str,
    new_fee: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::EosOnEth, signature, debug_command_hash))
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db)))
        .map(|_| sanity_check_basis_points_value(new_fee))
        .and_then(|_| EosEthTokenDictionary::get_from_db(db))
        .and_then(|dictionary| {
            dictionary.change_eth_fee_basis_points_and_update_in_db(db, &convert_hex_to_eth_address(address)?, new_fee)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true, "address": address, "new_fee": new_fee}).to_string())
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Set EOS Fee Basis Points
///
/// This function takes an address and a new fee param. It gets the `EosEthTokenDictionary` from
/// the database then finds the entry pertaining to the address in question and if successful,
/// updates the fee associated with that address before saving the dictionary back into the
/// database. If no entry is found for a given `address` the function will return an error saying
/// as such.
///
/// #### NOTE: Using a fee of 0 will mean no fees are taken.
pub fn debug_set_eos_fee_basis_points<D: DatabaseInterface>(
    db: &D,
    address: &str,
    new_fee: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db)))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::EosOnEth, signature, debug_command_hash))
        .map(|_| sanity_check_basis_points_value(new_fee))
        .and_then(|_| EosEthTokenDictionary::get_from_db(db))
        .and_then(|dictionary| {
            dictionary.change_eos_fee_basis_points_and_update_in_db(db, &EosAccountName::from_str(address)?, new_fee)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true, "address": address, "new_fee": new_fee}).to_string())
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Withwdraw Fees
///
/// This function takes an ETH address and uses it to search through the token dictionary to find a
/// corresponding entry. Once found, that entry's accrued fees are zeroed, a timestamp set in that
/// entry to mark the withdrawal date and the dictionary saved back in the database. Finally, an
/// EOS transaction is created to transfer the `<accrued_fees>` amount of tokens to the passed in
/// recipient address.
pub fn debug_withdraw_fees<D: DatabaseInterface>(
    db: &D,
    token_address: &str,
    recipient_address: &str,
    ref_block_num: u16,
    ref_block_prefix: u32,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let dictionary = EosEthTokenDictionary::get_from_db(db)?;
    let dictionary_entry_eth_address = convert_hex_to_eth_address(token_address)?;
    let eos_smart_contract_address = EosDbUtils::new(db).get_eos_account_name_from_db()?.to_string();
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db)))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::EosOnEth, signature, debug_command_hash))
        .and_then(|_| dictionary.withdraw_fees_and_save_in_db(db, &dictionary_entry_eth_address))
        .and_then(|(_, fee_amount)| {
            let amount = dictionary.convert_u256_to_eos_asset_string(&dictionary_entry_eth_address, &fee_amount)?;
            info!("Amount as EOS asset: {}", amount);
            let eos_action = EosAction::from_str(
                &eos_smart_contract_address,
                &PEGOUT_ACTION_NAME.into(),
                vec![PermissionLevel::from_str(
                    &eos_smart_contract_address,
                    &EOS_ACCOUNT_PERMISSION_LEVEL.into(),
                )?],
                PTokenPegOutAction::from_str(
                    &dictionary
                        .get_entry_via_eth_address(&dictionary_entry_eth_address)?
                        .eos_address,
                    &amount,
                    recipient_address,
                    &[],
                )?,
            )?;
            EosSignedTransaction::from_unsigned_tx(
                &eos_smart_contract_address,
                &amount,
                &EosDbUtils::new(db).get_eos_chain_id_from_db()?,
                &EosPrivateKey::get_from_db(db)?,
                &EosTransaction::new(
                    get_eos_tx_expiration_timestamp_with_offset(0u32)?,
                    ref_block_num,
                    ref_block_prefix,
                    vec![eos_action],
                ),
            )
        })
        .and_then(|eos_signed_tx| {
            db.end_transaction()?;
            Ok(json!({
                "success": true,
                "eos_tx_signature": eos_signed_tx.signature,
                "eos_serialized_tx": eos_signed_tx.transaction,
            })
            .to_string())
        })
}

/// # Debug Set Accrued Fees
///
/// This function updates the accrued fees value in the dictionary entry retrieved from the passed
/// in ETH address.
pub fn debug_set_accrued_fees_in_dictionary<D: DatabaseInterface>(
    db: &D,
    token_address: &str,
    fee_amount: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug setting accrued fees in dictionary...");
    let dictionary = EosEthTokenDictionary::get_from_db(db)?;
    let dictionary_entry_eth_address = convert_hex_to_eth_address(token_address)?;
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db)))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::EosOnEth, signature, debug_command_hash))
        .and_then(|_| {
            dictionary.set_accrued_fees_and_save_in_db(
                db,
                &dictionary_entry_eth_address,
                U256::from_dec_str(fee_amount)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true,"fee":fee_amount}).to_string())
}
