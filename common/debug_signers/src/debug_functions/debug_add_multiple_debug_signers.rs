use std::str::FromStr;

use common::{core_type::CoreType, errors::AppError, traits::DatabaseInterface, types::Result};
use common_eth::{convert_hex_to_eth_address, convert_hex_to_h256, EthSignature};
use derive_more::Deref;
use function_name::named;
use serde::Deserialize;
use serde_json::json;

#[cfg(not(feature = "no-safe-debug-signers"))]
use crate::SAFE_DEBUG_SIGNATORIES;
use crate::{DebugSignatories, DebugSignatory};

#[derive(Deserialize, Deref)]
struct DebugSignersJson(Vec<DebugSignerJson>);

impl DebugSignersJson {
    fn to_debug_signatories(&self) -> Result<Vec<DebugSignatory>> {
        self.iter()
            .map(|signer_json| {
                let eth_address = convert_hex_to_eth_address(&signer_json.eth_address)?;
                let debug_signatory = DebugSignatory::new(&signer_json.name, &eth_address);
                Ok(debug_signatory)
            })
            .collect::<Result<Vec<_>>>()
    }
}

impl FromStr for DebugSignersJson {
    type Err = AppError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(serde_json::from_str::<Vec<DebugSignerJson>>(s)?))
    }
}

#[derive(Deserialize)]
struct DebugSignerJson {
    name: String,
    eth_address: String,
}

impl FromStr for DebugSignerJson {
    type Err = AppError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

/// Debug Add Multiple Debug Signers
///
/// Adds new debug signatories to the list. Since this is a debug function, it requires a valid
/// signature from an address in the list of debug signatories. But because this list begins life
/// empty, we have a chicken and egg scenario. And so to solve this, if the addition is the _first_
/// one, we instead require a signature from the `SAFE_ETH_ADDRESS` in order to validate the
/// command.
#[named]
#[cfg(not(feature = "no-safe-debug-signers"))]
pub fn debug_add_multiple_debug_signers<D: DatabaseInterface>(
    db: &D,
    debug_signers_json: &str,
    core_type: &CoreType,
    signature_str: &str,
) -> Result<String> {
    info!("✔ Adding debug signer to list...");
    let debug_signatories_to_add = DebugSignersJson::from_str(debug_signers_json)?.to_debug_signatories()?;

    if !cfg!(feature = "skip-db-transactions") {
        db.start_transaction()?
    };

    DebugSignatories::get_from_db(db)
        .and_then(|debug_signatories| {
            let debug_command_hash = convert_hex_to_h256(&get_debug_command_hash!(
                function_name!(),
                debug_signers_json,
                core_type
            )()?)?;
            let signature = EthSignature::from_str(signature_str)?;

            if debug_signatories.is_empty() {
                info!("✔ Validating debug signers addition using the safe address...");
                SAFE_DEBUG_SIGNATORIES
                    .maybe_validate_signature_and_increment_nonce_in_db(db, core_type, &debug_command_hash, &signature)
                    .and_then(|_| debug_signatories.add_multi_and_update_in_db(db, &debug_signatories_to_add))
            } else {
                debug_signatories
                    .maybe_validate_signature_and_increment_nonce_in_db(db, core_type, &debug_command_hash, &signature)
                    .and_then(|_| DebugSignatories::get_from_db(db))
                    .and_then(|debug_signatories| {
                        debug_signatories.add_multi_and_update_in_db(db, &debug_signatories_to_add)
                    })
            }
        })
        .and_then(|_| {
            if !cfg!(feature = "skip-db-transactions") {
                db.end_transaction()
            } else {
                Ok(())
            }
        })
        .map(|_| {
            json!({
                "debug_add_multi_debug_signers_success":true,
                "signers_added": debug_signatories_to_add,
            })
            .to_string()
        })
}

/// Debug Add Multiple Debug Signers
///
/// Adds new debug signatories to the list. Since this is a debug function, it requires a valid
/// signature from an address in the list of debug signatories. But because this list begins life
/// empty, we have a chicken and egg scenario. And so to solve this, if the addition is the _first_
/// one, we instead require a signature from the `SAFE_ETH_ADDRESS` in order to validate the
/// command.
#[named]
#[cfg(feature = "no-safe-debug-signers")]
pub fn debug_add_multiple_debug_signers<D: DatabaseInterface>(
    db: &D,
    debug_signers_json: &str,
    core_type: &CoreType,
    signature_str: &str,
) -> Result<String> {
    info!("✔ Adding debug signer to list...");
    let debug_signatories_to_add = DebugSignersJson::from_str(debug_signers_json)?.to_debug_signatories()?;

    if !cfg!(feature = "skip-db-transactions") {
        db.start_transaction()?
    };

    DebugSignatories::get_from_db(db)
        .and_then(|debug_signatories| {
            let debug_command_hash = convert_hex_to_h256(&get_debug_command_hash!(
                function_name!(),
                debug_signers_json,
                core_type
            )()?)?;
            let signature = EthSignature::from_str(signature_str)?;

            if debug_signatories.is_empty() {
                info!("adding multiple debug signers to empty list without validating command signature...");
                debug_signatories.add_multi_and_update_in_db(db, &debug_signatories_to_add)
            } else {
                debug_signatories
                    .maybe_validate_signature_and_increment_nonce_in_db(db, core_type, &debug_command_hash, &signature)
                    .and_then(|_| DebugSignatories::get_from_db(db))
                    .and_then(|debug_signatories| {
                        debug_signatories.add_multi_and_update_in_db(db, &debug_signatories_to_add)
                    })
            }
        })
        .and_then(|_| {
            if !cfg!(feature = "skip-db-transactions") {
                db.end_transaction()
            } else {
                Ok(())
            }
        })
        .map(|_| {
            json!({
                "debug_add_multi_debug_signers_success":true,
                "signers_added": debug_signatories_to_add,
            })
            .to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_sample_signers_json_string() -> String {
        json!([
            {
                "name": "address1",
                "eth_address": "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
            },{
                "name": "address2",
                "eth_address": "0xb522f30ba03188d37893504d435beed000925485",
            }
        ])
        .to_string()
    }

    #[test]
    fn should_get_signers_json_from_str() {
        let s = get_sample_signers_json_string();
        let result = DebugSignersJson::from_str(&s);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_debug_signers_from_str() {
        let s = get_sample_signers_json_string();
        let result = DebugSignersJson::from_str(&s).unwrap().to_debug_signatories();
        assert!(result.is_ok());
    }
}
