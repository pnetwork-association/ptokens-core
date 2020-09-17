use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::eth::update_eth_linker_hash::maybe_update_eth_linker_hash,
};

pub fn maybe_update_eth_linker_hash_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe updating the ETH linker hash...");
    maybe_update_eth_linker_hash(&state.db).and(Ok(state))
}
