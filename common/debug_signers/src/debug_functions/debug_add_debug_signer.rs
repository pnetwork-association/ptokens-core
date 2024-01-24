use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::{convert_hex_to_eth_address, convert_hex_to_h256, EthSignature};
use function_name::named;
use serde_json::json;

use crate::{DebugSignatories, DebugSignatory, SAFE_DEBUG_SIGNATORIES};

/// Debug Add Debug Signer
///
/// Adds a debug signatory to the list. Since this is a debug function, it requires a valid
/// signature from an address in the list of debug signatories. But because this list begins life
/// empty, we have a chicken and egg scenario. And so to solve this, if the addition is the first_
/// one, we instead require a signature from the `SAFE_ETH_ADDRESS` in order to validate the
/// command. This requirement can be disabled with a passed in boolean.
#[named]
pub fn debug_add_debug_signer<D: DatabaseInterface>(
    db: &D,
    signatory_name: &str,
    eth_address_str: &str,
    core_type: &CoreType,
    signature_str: &str,
    use_safe_debug_signers: bool,
    use_db_tx: bool,
) -> Result<String> {
    info!("✔ Adding debug signer to list...");
    let eth_address = convert_hex_to_eth_address(eth_address_str)?;
    if use_db_tx {
        db.start_transaction()?
    };

    DebugSignatories::get_from_db(db)
        .and_then(|debug_signatories| {
            let debug_command_hash = convert_hex_to_h256(&get_debug_command_hash!(
                function_name!(),
                signatory_name,
                eth_address_str,
                core_type
            )()?)?;
            let signature = EthSignature::from_str(signature_str)?;
            let debug_signatory_to_add = DebugSignatory::new(signatory_name, &eth_address);

            if debug_signatories.is_empty() {
                let msg = "validating the debug signer addition using the safe address...";
                if use_safe_debug_signers {
                    debug!("{msg}");
                    SAFE_DEBUG_SIGNATORIES
                        .maybe_validate_signature_and_increment_nonce_in_db(
                            db,
                            core_type,
                            &debug_command_hash,
                            &signature,
                        )
                        .and_then(|_| debug_signatories.add_and_update_in_db(db, &debug_signatory_to_add))
                } else {
                    debug!("not {msg}");
                    debug_signatories.add_and_update_in_db(db, &debug_signatory_to_add)
                }
            } else {
                debug_signatories
                    .maybe_validate_signature_and_increment_nonce_in_db(db, core_type, &debug_command_hash, &signature)
                    .and_then(|_| DebugSignatories::get_from_db(db))
                    .and_then(|debug_signatories| debug_signatories.add_and_update_in_db(db, &debug_signatory_to_add))
            }
        })
        .and_then(|_| if use_db_tx { db.end_transaction() } else { Ok(()) })
        .map(|_| json!({"debugAddSignatorySuccess":true, "ethAddress": eth_address_str}).to_string())
}
