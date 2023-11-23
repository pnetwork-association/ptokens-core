use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::convert_hex_to_h256;
use serde_json::Value as JsonValue;

use crate::DebugSignatories;
#[cfg(not(feature = "no-safe-debug-signers"))]
use crate::SAFE_DEBUG_SIGNATORIES;

/// Get Debug Signature Info
///
/// Gets the information required to sign a valid debug function signaure, require in order to run
/// a debug function. The `debug_command_hash_str` is a hash of the `CLI_ARGS` struct populated
/// with the arguments required to run the desired debug function.
#[cfg(not(feature = "no-safe-debug-signers"))]
pub fn get_debug_signature_info<D: DatabaseInterface>(
    db: &D,
    core_type: &CoreType,
    debug_command_hash_str: &str,
) -> Result<JsonValue> {
    if !cfg!(feature = "skip-db-transactions") {
        db.start_transaction()?
    };

    DebugSignatories::get_from_db(db).and_then(|debug_signatories| {
        if !cfg!(feature = "skip-db-transactions") {
            db.end_transaction()?
        };
        let debug_command_hash = convert_hex_to_h256(debug_command_hash_str)?;
        if debug_signatories.is_empty() {
            // NOTE: If there are no signers yet, we show the safe address signing info, since
            // with that, new debug signers can be added.
            SAFE_DEBUG_SIGNATORIES.to_signature_info_json(core_type, &debug_command_hash, None)
        } else {
            debug_signatories.to_signature_info_json(core_type, &debug_command_hash, None)
        }
    })
}

/// Get Debug Signature Info
///
/// Gets the information required to sign a valid debug function signaure, require in order to run
/// a debug function. The `debug_command_hash_str` is a hash of the `CLI_ARGS` struct populated
/// with the arguments required to run the desired debug function.
#[cfg(feature = "no-safe-debug-signers")]
pub fn get_debug_signature_info<D: DatabaseInterface>(
    db: &D,
    core_type: &CoreType,
    debug_command_hash_str: &str,
) -> Result<JsonValue> {
    if !cfg!(feature = "skip-db-transactions") {
        db.start_transaction()?
    };
    DebugSignatories::get_from_db(db).and_then(|debug_signatories| {
        if !cfg!(feature = "skip-db-transactions") {
            db.end_transaction()?
        };
        let debug_command_hash = convert_hex_to_h256(debug_command_hash_str)?;
        if debug_signatories.is_empty() {
            // NOTE: If there are no signers yet we return the info from an emptry set of
            // signers. This includes a message telling the user to add a debug signer.
            DebugSignatories::default().to_signature_info_json(core_type, &debug_command_hash, None)
        } else {
            debug_signatories.to_signature_info_json(core_type, &debug_command_hash, None)
        }
    })
}
