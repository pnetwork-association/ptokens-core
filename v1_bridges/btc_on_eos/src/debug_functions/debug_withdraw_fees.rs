use common::{traits::DatabaseInterface, types::Result, utils::prepend_debug_output_marker_to_string};
use common_btc::{
    extract_utxos_from_p2pkh_txs,
    get_hex_tx_from_signed_btc_tx,
    get_pay_to_pub_key_hash_script,
    save_utxos_to_db,
    BtcDbUtils,
};
use common_debug_signers::validate_debug_command_signature;
use common_fees::get_btc_on_eos_fee_withdrawal_tx;
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

/// # Debug Get Fee Withdrawal Tx
///
/// This function crates a BTC transaction to the passed in address for the amount of accrued fees
/// accounted for in the encrypted database. The function then reset this value back to zero. The
/// signed transaction is returned to the caller.
#[named]
pub fn debug_get_fee_withdrawal_tx<D: DatabaseInterface>(db: &D, btc_address: &str, signature: &str) -> Result<String> {
    info!("✔ Debug getting `BtcOnEos` withdrawal tx...");
    let btc_db_utils = BtcDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), btc_address)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| get_btc_on_eos_fee_withdrawal_tx(db, btc_address))
        .and_then(|btc_tx| {
            let change_utxos = get_pay_to_pub_key_hash_script(&btc_db_utils.get_btc_address_from_db()?)
                .map(|target_script| extract_utxos_from_p2pkh_txs(&target_script, &[btc_tx.clone()]))?;
            save_utxos_to_db(db, &change_utxos)?;
            db.end_transaction()?;
            Ok(json!({ "signed_btc_tx": get_hex_tx_from_signed_btc_tx(&btc_tx) }).to_string())
        })
        .map(prepend_debug_output_marker_to_string)
}
