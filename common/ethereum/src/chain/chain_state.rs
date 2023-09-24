use std::fmt;

use common_metadata::MetadataChainId;
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use super::{Chain, ChainError};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainState {
    pnetwork_hub: EthAddress,
    confirmations: u64,
    chain_id: MetadataChainId,
    latest_block_num: u64,
    latest_block_hash: EthHash,
    tail_block_num: u64,
    tail_block_hash: EthHash,
    canon_block_num: u64,
    canon_block_hash: EthHash,
}

impl From<&ChainState> for Json {
    fn from(c: &ChainState) -> Json {
        json!(c)
    }
}

impl TryFrom<&ChainState> for Vec<u8> {
    type Error = ChainError;

    fn try_from(c: &ChainState) -> Result<Vec<u8>, Self::Error> {
        Ok(serde_json::to_vec(c)?)
    }
}

impl fmt::Display for ChainState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error convert `ChainState` to pretty json string: {e}",),
        }
    }
}

impl TryFrom<&Chain> for ChainState {
    type Error = ChainError;

    fn try_from(c: &Chain) -> Result<ChainState, Self::Error> {
        let pnetwork_hub = *c.hub();
        let chain_id = *c.chain_id();
        let confirmations = *c.confirmations();

        let maybe_latest_block_data = c.get_latest_block_data();
        // NOTE: We can't really know which of the block data is canonical at this point".
        let latest_block_num = maybe_latest_block_data.map(|d| *d[0].number()).unwrap_or_default();
        let latest_block_hash = maybe_latest_block_data
            .map(|d| *d[0].hash())
            .unwrap_or_else(EthHash::zero);

        let maybe_canon_block_data = c.get_canon_block_data();
        // FIXME Walk this one back to find out what we're classing as canonical here.
        let canon_block_num = maybe_canon_block_data.map(|d| *d[0].number()).unwrap_or_default();
        let canon_block_hash = maybe_canon_block_data
            .map(|d| *d[0].hash())
            .unwrap_or_else(EthHash::zero);

        let maybe_tail_block_data = c.get_tail_block_data();
        // NOTE: We can't really know which of the block data is canonical at this point".
        let tail_block_num = maybe_tail_block_data.map(|d| *d[0].number()).unwrap_or_default();
        let tail_block_hash = maybe_tail_block_data
            .map(|d| *d[0].hash())
            .unwrap_or_else(EthHash::zero);

        let r = ChainState {
            pnetwork_hub,
            confirmations,
            chain_id,
            latest_block_num,
            latest_block_hash,
            tail_block_num,
            tail_block_hash,
            canon_block_num,
            canon_block_hash,
        };

        Ok(r)
    }
}
