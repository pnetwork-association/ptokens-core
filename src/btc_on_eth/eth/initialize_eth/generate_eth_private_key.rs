use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::eth_crypto::eth_private_key::EthPrivateKey,
    btc_on_eth::eth::{
        eth_state::EthState,
        eth_database_utils::put_eth_private_key_in_db,
    },
};

pub fn generate_and_store_eth_private_key<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Generating & storing ETH private key...");
    put_eth_private_key_in_db(
        &state.db,
        &EthPrivateKey::generate_random()?,
    )
        .map(|_| state)
}
