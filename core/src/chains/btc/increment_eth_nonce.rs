use crate::{
    chains::eth::eth_database_utils::EthDbUtilsExt,
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_increment_eth_nonce_in_db<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    if state.use_any_sender_tx_type() {
        info!("✔ Not incrementing ETH account nonce due to using AnySender transactions instead!");
        return Ok(state);
    }
    match state.get_eth_signed_txs() {
        Err(_) => {
            info!("✔ Not incrementing ETH account nonce - no signatures made!");
            Ok(state)
        },
        Ok(signed_txs) => {
            info!("✔ Incrementing ETH account nonce by {}", signed_txs.len());
            state
                .eth_db_utils
                .increment_eth_account_nonce_in_db(signed_txs.len() as u64)
                .and(Ok(state))
        },
    }
}
