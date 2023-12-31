use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_chain_ids::EthChainId;
use common_eth::{
    convert_hex_to_eth_address,
    end_eth_db_transaction_and_return_state,
    initialize_eth_core_with_no_contract_tx,
    start_eth_db_transaction_and_return_state,
    EthDbUtilsExt,
    EthInitializationOutput,
    EthState,
    ETH_CORE_IS_INITIALIZED_JSON,
};

pub fn maybe_initialize_eth_enclave<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    confs: u64,
    erc777_contract_address: &str,
) -> Result<String> {
    if CoreType::host_core_is_initialized(db) {
        Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        let is_native = false;
        start_eth_db_transaction_and_return_state(EthState::init(db))
            .and_then(|state| {
                initialize_eth_core_with_no_contract_tx(
                    block_json,
                    &EthChainId::try_from(chain_id)?,
                    gas_price,
                    confs,
                    state,
                    is_native,
                )
            })
            .and_then(|state| {
                state
                    .eth_db_utils
                    .put_btc_on_int_smart_contract_address_in_db(&convert_hex_to_eth_address(
                        erc777_contract_address,
                    )?)?;
                Ok(state)
            })
            .and_then(end_eth_db_transaction_and_return_state)
            .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils))
    }
}
