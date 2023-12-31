use common::{
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use serde_json::json;

use crate::utxo_manager::put_utxo_nonce_in_db;

/// # Debug Set BTC UTXO Nonce
///
/// This function set to the given value BTC UTXO nonce in the encryped database.
#[named]
pub fn debug_set_btc_utxo_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting BTC UTXO nonce...");
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &new_nonce, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
        .and_then(|_| put_utxo_nonce_in_db(db, new_nonce))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"set_btc_utxo_nonce":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use common::test_utils::{get_test_database, DUMMY_DEBUG_COMMAND_SIGNATURE};

    use super::*;
    use crate::utxo_manager::get_utxo_nonce_from_db;

    #[test]
    fn should_set_btc_utxo_nonce() {
        let db = get_test_database();
        let nonce = 6;
        put_utxo_nonce_in_db(&db, nonce).unwrap();
        assert_eq!(get_utxo_nonce_from_db(&db).unwrap(), nonce);
        let new_nonce = 4;
        debug_set_btc_utxo_nonce(&db, new_nonce, &CoreType::BtcOnInt, DUMMY_DEBUG_COMMAND_SIGNATURE).unwrap();
        assert_eq!(get_utxo_nonce_from_db(&db).unwrap(), new_nonce);
    }
}
