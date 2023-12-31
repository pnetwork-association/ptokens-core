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
use ethereum_types::Address as EthAddress;

pub fn init_int_core<D: DatabaseInterface>(
    state: EthState<D>,
    block_json: &str,
    chain_id: u64,
    gas_price: u64,
    canon_to_tip_length: u64,
    erc777_contract_address: &EthAddress,
    router_contract_address: &EthAddress,
) -> Result<String> {
    let is_native = false;
    start_eth_db_transaction_and_return_state(state)
        .and_then(|state| {
            initialize_eth_core_with_no_contract_tx(
                block_json,
                &EthChainId::try_from(chain_id)?,
                gas_price,
                canon_to_tip_length,
                state,
                is_native,
            )
        })
        .and_then(|state| {
            state
                .eth_db_utils
                .put_eth_router_smart_contract_address_in_db(router_contract_address)?;
            state
                .eth_db_utils
                .put_btc_on_int_smart_contract_address_in_db(erc777_contract_address)?;
            Ok(state)
        })
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils))
}

pub fn maybe_initialize_int_core<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    chain_id: u64,
    gas_price: u64,
    canon_to_tip_length: u64,
    erc777_contract_address: &str,
    router_contract_address: &str,
) -> Result<String> {
    let state = EthState::init(db);
    if CoreType::host_core_is_initialized(db) {
        Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        init_int_core(
            state,
            block_json,
            chain_id,
            gas_price,
            canon_to_tip_length,
            &convert_hex_to_eth_address(erc777_contract_address)?,
            &convert_hex_to_eth_address(router_contract_address)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;
    use common_eth::{convert_hex_to_eth_address, EthState};

    use super::*;
    use crate::test_utils::get_sample_int_submission_material_json_str_n;

    #[test]
    fn should_init_int_core() {
        let db = get_test_database();
        let eth_block_0 = get_sample_int_submission_material_json_str_n(0);
        let eth_state = EthState::init(&db);
        let eth_chain_id = 3;
        let eth_gas_price = 20_000_000_000;
        let eth_canon_to_tip_length = 2;
        let ptoken_address_hex = "0x0f513aA8d67820787A8FDf285Bfcf967bF8E4B8b";
        let ptoken_address = convert_hex_to_eth_address(ptoken_address_hex).unwrap();
        let router_address_hex = "0x88d19e08cd43bba5761c10c588b2a3d85c75041f";
        let router_address = convert_hex_to_eth_address(router_address_hex).unwrap();
        let result = init_int_core(
            eth_state,
            &eth_block_0,
            eth_chain_id,
            eth_gas_price,
            eth_canon_to_tip_length,
            &ptoken_address,
            &router_address,
        );
        assert!(result.is_ok());
    }
}
