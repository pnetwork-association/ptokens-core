use std::result::Result;

use common::{CoreType, DatabaseInterface};
use common_enclave_info::EnclaveInfo;
use common_eth::{EthDbUtils, EthDbUtilsExt, EvmDbUtils, HostCoreState, NativeCoreState, VaultUsingCores};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::SentinelError;

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreState {
    info: EnclaveInfo,
    host: HostCoreState,
    native: NativeCoreState,
}

impl CoreState {
    pub fn get<D: DatabaseInterface>(db: &D, core_type: &CoreType) -> Result<Self, SentinelError> {
        let eth_db_utils = EthDbUtils::new(db);
        let evm_db_utils = EvmDbUtils::new(db);

        Ok(Self {
            info: EnclaveInfo::new(eth_db_utils.get_db())?,
            host: HostCoreState::new(
                &evm_db_utils,
                &VaultUsingCores::from_core_type(core_type)?.get_vault_contract(&evm_db_utils)?,
                Some(evm_db_utils.get_eth_router_smart_contract_address_from_db()?),
            )?,
            native: NativeCoreState::new(
                &eth_db_utils,
                &VaultUsingCores::from_core_type(core_type)?.get_vault_contract(&eth_db_utils)?,
                Some(eth_db_utils.get_eth_router_smart_contract_address_from_db()?),
            )?,
        })
    }

    fn to_json(&self) -> JsonValue {
        json!(self)
    }
}

impl std::fmt::Display for CoreState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_json())
    }
}
