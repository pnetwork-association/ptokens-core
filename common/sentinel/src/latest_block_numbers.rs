use common_chain_ids::EthChainId;
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{SentinelError, WebSocketMessagesEncodable};

#[derive(Clone, Debug, Deref, Constructor, Serialize, Deserialize)]
pub struct LatestBlockNumbers(Vec<LatestBlockNumber>);

#[derive(Clone, Debug, Deref, Constructor, Serialize, Deserialize)]
pub struct LatestBlockNumber((EthChainId, u64));

impl LatestBlockNumber {
    pub fn eth_chain_id(&self) -> &EthChainId {
        &self.0 .0
    }

    pub fn latest_block_number(&self) -> u64 {
        self.0 .1
    }
}

impl LatestBlockNumbers {
    pub fn get_for(&self, needle: &EthChainId) -> Result<u64, SentinelError> {
        let r = self.iter().fold(None, |mut res, structure| {
            if structure.eth_chain_id() == needle {
                res = Some(structure.latest_block_number());
            }
            res
        });
        match r {
            Some(n) => Ok(n),
            _ => Err(SentinelError::NoLatestBlockNumber(needle.clone())),
        }
    }
}

impl TryFrom<WebSocketMessagesEncodable> for LatestBlockNumbers {
    type Error = SentinelError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        debug!("trying to get `LatestBlockNumbers` from `WebSocketMessagesEncodable`...");
        let j = Json::try_from(m)?;
        Ok(serde_json::from_value(j)?)
    }
}
