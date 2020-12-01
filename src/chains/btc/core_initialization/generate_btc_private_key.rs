use crate::{
    chains::btc::{
        btc_crypto::btc_private_key::BtcPrivateKey,
        btc_database_utils::put_btc_private_key_in_db,
        btc_state::BtcState,
        core_initialization::btc_init_utils::get_btc_network_from_arg,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn generate_and_store_btc_private_key<D>(network: &str, state: BtcState<D>) -> Result<BtcState<D>>
where
    D: DatabaseInterface,
{
    info!("✔ Generating & storing BTC private key...");
    put_btc_private_key_in_db(
        &state.db,
        &BtcPrivateKey::generate_random(get_btc_network_from_arg(network))?,
    )
    .and(Ok(state))
}