use std::result::Result;

use common::DatabaseInterface;
use common_eth::{ChainDbUtils, ChainError, ChainState};
use common_metadata::MetadataChainId;
use common_network_ids::{NetworkId, NetworkIdError};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::SentinelError;

#[derive(Debug, Deserialize, Serialize)]
pub struct CoreState {
    chain_state: Vec<ChainState>,
}

impl CoreState {
    pub fn get<D: DatabaseInterface>(db: &D, network_ids: Vec<NetworkId>) -> Result<Self, SentinelError> {
        let mcids = network_ids
            .iter()
            .map(MetadataChainId::try_from)
            .collect::<Result<Vec<MetadataChainId>, NetworkIdError>>()?;
        let chain_db_utils = ChainDbUtils::new(db);
        let chain_state = mcids
            .iter()
            .map(|mcid| ChainState::new(&chain_db_utils, mcid))
            .collect::<Result<Vec<ChainState>, ChainError>>()
            .map_err(SentinelError::ChainError)?;

        Ok(Self { chain_state })
    }
}

impl std::fmt::Display for CoreState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self))
    }
}
