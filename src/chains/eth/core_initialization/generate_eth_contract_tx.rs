use crate::{
    chains::eth::{
        eth_crypto::eth_transaction::get_signed_ptoken_smart_contract_tx,
        eth_database_utils::get_eth_private_key_from_db,
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn generate_eth_contract_tx_and_put_in_state<D>(
    chain_id: u8,
    gas_price: u64,
    bytecode_path: &str,
    state: EthState<D>,
) -> Result<EthState<D>>
where
    D: DatabaseInterface,
{
    get_eth_private_key_from_db(&state.db)
        .and_then(|eth_private_key| {
            get_signed_ptoken_smart_contract_tx(
                0, // NOTE: ∵ first tx!
                chain_id,
                eth_private_key,
                gas_price,
                bytecode_path,
            )
        })
        .and_then(|signed_tx| state.add_misc_string_to_state(signed_tx))
}
