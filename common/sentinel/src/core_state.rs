use std::result::Result;

use common::{CoreType, DatabaseInterface};
use common_enclave_info::EnclaveInfo;
use common_eth::{EthDbUtils, EvmDbUtils, HostCoreState, NativeCoreState};
use ethereum_types::Address as EthAddress;
use serde::Serialize;
use serde_json::json;

use crate::SentinelError;

#[derive(Debug, Serialize)]
pub struct CoreState {
    info: EnclaveInfo,
    host: HostCoreState,
    native: NativeCoreState,
}

impl CoreState {
    pub fn get<D: DatabaseInterface>(db: &D) -> Result<Self, SentinelError> {
        let eth_db_utils = EthDbUtils::new(db);
        let evm_db_utils = EvmDbUtils::new(db);

        Ok(Self {
            host: HostCoreState::new(&evm_db_utils, &EthAddress::zero(), None)?,
            native: NativeCoreState::new(&eth_db_utils, &EthAddress::zero(), None)?,
            info: EnclaveInfo::new_with_core_type(eth_db_utils.get_db(), CoreType::V3Strongbox)?,
        })
    }
}

impl std::fmt::Display for CoreState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self))
    }
}
