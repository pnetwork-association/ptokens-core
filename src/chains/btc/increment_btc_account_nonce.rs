use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::eos_state::EosState,
    btc_on_eos::btc::btc_database_utils::{
        put_btc_account_nonce_in_db,
        get_btc_account_nonce_from_db,
    },
};

pub fn increment_btc_account_nonce<D>(
    db: &D,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()>
    where D: DatabaseInterface
{
    let new_nonce = num_signatures + current_nonce;
    info!("✔ Incrementing btc account nonce by {} nonce from {} to {}", num_signatures, current_nonce, new_nonce);
    put_btc_account_nonce_in_db(db, new_nonce)
}

pub fn maybe_increment_btc_signature_nonce_and_return_eos_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    let num_txs = &state.signed_txs.len();
    match num_txs {
        0 => {
            info!("✔ No signatures in state ∴ not incrementing nonce");
            Ok(state)
        }
        _ => {
            increment_btc_account_nonce(&state.db, get_btc_account_nonce_from_db(&state.db)?, *num_txs as u64)
                .and(Ok(state))
        }
    }
}
