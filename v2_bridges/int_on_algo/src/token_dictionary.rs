use common::{dictionaries::evm_algo::dictionary::EvmAlgoTokenDictionary, traits::DatabaseInterface, types::Result};
use common_algo::AlgoState;

pub fn get_evm_algo_token_dictionary_and_add_to_algo_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Getting `EvmAlgoTokenDictionary` and adding to ALGO state...");
    EvmAlgoTokenDictionary::get_from_db(state.algo_db_utils.get_db())
        .and_then(|dictionary| state.add_evm_algo_dictionary(dictionary))
}
