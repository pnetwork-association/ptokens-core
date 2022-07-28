use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        algo::{algo_database_utils::AlgoDbUtils, algo_enclave_state::AlgoEnclaveState},
        eth::{
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_enclave_state::EthEnclaveState,
        },
    },
    dictionaries::evm_algo::EvmAlgoTokenDictionary,
    enclave_info::EnclaveInfo,
    int_on_algo::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    int: EthEnclaveState,
    algo: AlgoEnclaveState,
    dictionary: EvmAlgoTokenDictionary,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, algo_db_utils: &AlgoDbUtils<D>) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(eth_db_utils.get_db()),
            algo: AlgoEnclaveState::new(algo_db_utils)?,
            dictionary: EvmAlgoTokenDictionary::get_from_db(algo_db_utils.get_db())?,
            int: EthEnclaveState::new(
                eth_db_utils,
                &eth_db_utils.get_int_on_algo_smart_contract_address()?,
                Some(eth_db_utils.get_eth_router_smart_contract_address_from_db()?),
            )?,
        })
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// # Get Enclave State
///
/// This function returns a JSON containing the enclave state, including state relevant to each
/// blockchain controlled by this core.
pub fn get_enclave_state<D: DatabaseInterface>(db: D) -> Result<String> {
    info!("✔ Getting enclave state...");
    let eth_db_utils = EthDbUtils::new(&db);
    let algo_db_utils = AlgoDbUtils::new(&db);
    check_core_is_initialized(&eth_db_utils, &algo_db_utils)
        .and_then(|_| EnclaveState::new(&eth_db_utils, &algo_db_utils)?.to_string())
}
