use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_crypto::eth_transaction::get_signed_ptoken_smart_contract_tx,
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

fn generate_contract_tx_and_put_in_state<'a, D: DatabaseInterface>(
    chain_id: &EthChainId,
    gas_price: u64,
    bytecode_path: &str,
    state: EthState<'a, D>,
    is_for_eth: bool,
) -> Result<EthState<'a, D>> {
    let private_key = if is_for_eth {
        state.eth_db_utils.get_eth_private_key_from_db()
    } else {
        state.evm_db_utils.get_eth_private_key_from_db()
    }?;
    get_signed_ptoken_smart_contract_tx(0, chain_id, &private_key, gas_price, bytecode_path)
        .and_then(|signed_tx| state.add_misc_string_to_state(signed_tx))
}

pub fn generate_eth_contract_tx_and_put_in_state<'a, D: DatabaseInterface>(
    chain_id: &EthChainId,
    gas_price: u64,
    bytecode_path: &str,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    generate_contract_tx_and_put_in_state(chain_id, gas_price, bytecode_path, state, true)
}

pub fn generate_evm_contract_tx_and_put_in_state<'a, D: DatabaseInterface>(
    chain_id: &EthChainId,
    gas_price: u64,
    bytecode_path: &str,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    generate_contract_tx_and_put_in_state(chain_id, gas_price, bytecode_path, state, false)
}
