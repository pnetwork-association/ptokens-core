use serde_json::json;

use crate::{
    chains::{
        btc::{btc_database_utils::BtcDatabaseKeysJson, utxo_manager::utxo_constants::get_utxo_constants_db_keys},
        eth::eth_database_utils::EthDatabaseKeysJson,
    },
    constants::DB_KEY_PREFIX,
    debug_functions::DEBUG_SIGNATORIES_DB_KEY,
    types::Result,
};

/// # Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn get_all_db_keys() -> Result<String> {
    Ok(json!({
        "btc": BtcDatabaseKeysJson::new(),
        "eth": EthDatabaseKeysJson::new(),
        "db_key_prefix": DB_KEY_PREFIX.to_string(),
        "utxo_manager": get_utxo_constants_db_keys(),
        "debug_signatories": format!("0x{}", hex::encode(&*DEBUG_SIGNATORIES_DB_KEY)),
    })
    .to_string())
}
