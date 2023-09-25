use std::result::Result;

use common::{CoreType, DatabaseInterface};
use common_enclave_info::EnclaveInfo;
use common_eth::{ChainDbUtils, ChainError, ChainState, EthDbUtils, EvmDbUtils, HostCoreState, NativeCoreState};
use common_metadata::MetadataChainId;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::SentinelError;

#[derive(Debug, Deserialize, Serialize)]
pub struct CoreState {
    info: EnclaveInfo,
    host: HostCoreState,
    native: NativeCoreState,
    chain_state: Vec<ChainState>,
}

impl CoreState {
    pub fn get<D: DatabaseInterface>(db: &D, mcids: Vec<MetadataChainId>) -> Result<Self, SentinelError> {
        let chain_db_utils = ChainDbUtils::new(db);
        let eth_db_utils = EthDbUtils::new(db);
        let evm_db_utils = EvmDbUtils::new(db);
        let chain_state = mcids
            .iter()
            .map(|mcid| ChainState::new(&chain_db_utils, mcid))
            .collect::<Result<Vec<ChainState>, ChainError>>()
            .map_err(SentinelError::ChainError)?;

        Ok(Self {
            chain_state,
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
