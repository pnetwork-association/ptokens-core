use serde::{Deserialize, Serialize};

use super::constants::CORE_TYPE;
use crate::{
    chains::{
        eos::{eos_database_utils::EosDbUtils, eos_enclave_state::EosEnclaveState},
        eth::{
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_enclave_state::EthEnclaveState,
        },
    },
    core_type::CoreType,
    enclave_info::EnclaveInfo,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    eth: EthEnclaveState,
    eos: EosEnclaveState,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, eos_db_utils: &EosDbUtils<D>) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(eth_db_utils.get_db())?,
            eth: EthEnclaveState::new(
                eth_db_utils,
                &eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
                None,
            )?,
            eos: EosEnclaveState::new_without_account_name(eos_db_utils)?,
        })
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// # Get Enclave State
///
/// This function returns a JSON containing the enclave state, including state relevant to each
/// blockchain controlled by this instance.
pub fn get_enclave_state<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting enclave state for {}...", CORE_TYPE);
    CoreType::check_is_initialized(db)
        .and_then(|_| EnclaveState::new(&EthDbUtils::new(db), &EosDbUtils::new(db))?.to_string())
}
