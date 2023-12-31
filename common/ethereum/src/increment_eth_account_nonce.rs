use common::{traits::DatabaseInterface, types::Result};

use crate::{
    eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    EthState,
};

fn increment_eth_account_nonce<D: DatabaseInterface>(
    eth_db_utils: &EthDbUtils<D>,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()> {
    let new_nonce = num_signatures + current_nonce;
    info!(
        "✔ Incrementing ETH account nonce by {} from {} to {}",
        num_signatures, current_nonce, new_nonce
    );
    eth_db_utils.put_eth_account_nonce_in_db(new_nonce)
}

pub fn maybe_increment_eth_account_nonce_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    let num_txs;
    if !state.erc20_on_evm_eth_signed_txs.is_empty() {
        info!("✔`'erc20-on-evm' ETH txs found!");
        num_txs = state.erc20_on_evm_eth_signed_txs.len();
    } else if !state.erc20_on_int_eth_signed_txs.is_empty() {
        info!("✔`'erc20-on-int' ETH txs found!");
        num_txs = state.erc20_on_int_eth_signed_txs.len();
    } else {
        num_txs = 0;
    }
    if num_txs == 0 {
        info!("✔ No signatures in state ∴ not incrementing ETH account nonce");
        Ok(state)
    } else {
        increment_eth_account_nonce(
            &state.eth_db_utils,
            state.eth_db_utils.get_eth_account_nonce_from_db()?,
            num_txs as u64,
        )
        .and(Ok(state))
    }
}
