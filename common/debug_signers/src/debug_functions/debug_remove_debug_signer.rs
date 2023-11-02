use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::{convert_hex_to_eth_address, convert_hex_to_h256, EthSignature};
use function_name::named;
use serde_json::json;

use crate::DebugSignatories;

/// Debug Remove Debug Signer
///
/// Removes a debug signatory from the list. Requires a valid signature from an existing debug
/// signatory in order to do so. If the supplied eth address is not in the list of debug
/// debug_signatories, nothing is removed.
#[named]
pub fn debug_remove_debug_signer<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    core_type: &CoreType,
    signature_str: &str,
) -> Result<String> {
    if !cfg!(feature = "skip-db-transaction") {
        db.start_transaction()?
    };

    DebugSignatories::get_from_db(db)
        .and_then(|debug_signatories| {
            let signature = EthSignature::from_str(signature_str)?;
            let eth_address = convert_hex_to_eth_address(eth_address_str)?;
            let debug_command_hash =
                convert_hex_to_h256(&get_debug_command_hash!(function_name!(), eth_address_str, core_type)()?)?;
            debug_signatories
                .maybe_validate_signature_and_increment_nonce_in_db(db, core_type, &debug_command_hash, &signature)
                .and_then(|_| DebugSignatories::get_from_db(db))
                .and_then(|debug_signatories| debug_signatories.remove_and_update_in_db(db, &eth_address))
        })
        .and_then(|_| {
            if !cfg!(feature = "skip-db-transaction") {
                db.end_transaction()
            } else {
                Ok(())
            }
        })
        .map(|_| json!({"debug_remove_signatory_success":true, "eth_address": eth_address_str}).to_string())
}
