use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_enclave_info::EnclaveInfo;
use common_eos::{EosDbUtils, EosEnclaveState};
use common_eth::{EthDbUtils, EthDbUtilsExt, EthEnclaveState};
use serde::{Deserialize, Serialize};

use crate::constants::CORE_TYPE;

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    eos: EosEnclaveState,
    eth: EthEnclaveState,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, eos_db_utils: &EosDbUtils<D>) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(eth_db_utils.get_db())?,
            eos: EosEnclaveState::new(eos_db_utils)?,
            eth: EthEnclaveState::new(
                eth_db_utils,
                &eth_db_utils.get_eos_on_eth_smart_contract_address_from_db()?,
                None,
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
/// blockchain controlled by this instance.
pub fn get_enclave_state<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting {} core state...", CORE_TYPE);
    CoreType::check_is_initialized(db)
        .and_then(|_| EnclaveState::new(&EthDbUtils::new(db), &EosDbUtils::new(db))?.to_string())
}
