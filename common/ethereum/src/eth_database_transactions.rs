use common::{traits::DatabaseInterface, types::Result};

use crate::EthState;

pub fn start_eth_db_transaction_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    state.eth_db_utils.get_db().start_transaction().map(|_| {
        info!("✔ ETH database transaction begun!");
        state
    })
}

pub fn end_eth_db_transaction_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    state.eth_db_utils.get_db().end_transaction().map(|_| {
        info!("✔ Eth database transaction ended!");
        state
    })
}
