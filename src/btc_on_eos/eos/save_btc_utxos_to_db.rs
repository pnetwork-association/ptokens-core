use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chain::btc::utxo_manager::utxo_database_utils::save_utxos_to_db,
};

pub fn maybe_save_btc_utxos_to_db<D>(state: EosState<D>) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe saving BTC UTXOs...");
    match &state.btc_utxos_and_values {
        None => {
            info!("✔ No BTC UTXOs in state to save!");
            Ok(state)
        }
        Some(utxos) => {
            save_utxos_to_db(&state.db, &utxos)
                .and_then(|_| Ok(state))
        }
    }
}
