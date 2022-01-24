use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn start_algo_db_transaction_and_return_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    state.algo_db_utils.get_db().start_transaction().map(|_| {
        info!("✔ ETH database transaction begun!");
        state
    })
}

pub fn end_algo_db_transaction_and_return_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    state.algo_db_utils.get_db().end_transaction().map(|_| {
        info!("✔ Eth database transaction ended!");
        state
    })
}
