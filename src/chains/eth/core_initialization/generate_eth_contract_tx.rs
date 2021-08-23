use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_crypto::eth_transaction::get_signed_ptoken_smart_contract_tx,
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn generate_eth_contract_tx_and_put_in_state<'a, D: DatabaseInterface>(
    chain_id: &EthChainId,
    gas_price: u64,
    bytecode_path: &str,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    state
        .eth_db_utils
        .get_eth_private_key_from_db()
        .and_then(|ref eth_private_key| {
            get_signed_ptoken_smart_contract_tx(0, chain_id, eth_private_key, gas_price, bytecode_path)
        })
        .and_then(|signed_tx| state.add_misc_string_to_state(signed_tx))
}
