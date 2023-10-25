use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{NetworkId, SentinelError, WebSocketMessagesEncodable};

#[derive(Clone, Debug, Deref, Constructor, Serialize, Deserialize)]
pub struct LatestBlockInfos(Vec<LatestBlockInfo>);

#[derive(Clone, Debug, Constructor, Serialize, Deserialize, Getters)]
pub struct LatestBlockInfo {
    block_number: u64,
    block_timestamp: u64,
    network_id: NetworkId,
}

impl LatestBlockInfos {
    pub fn get_for(&self, needle: &NetworkId) -> Result<LatestBlockInfo, SentinelError> {
        let r = self.iter().fold(None, |mut res, structure| {
            if structure.network_id() == needle {
                res = Some(structure);
            }
            res
        });
        match r {
            Some(n) => Ok(n.clone()),
            _ => Err(SentinelError::NoLatestBlockInfo(*needle)),
        }
    }
}

impl TryFrom<WebSocketMessagesEncodable> for LatestBlockInfos {
    type Error = SentinelError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        debug!("trying to get `LatestBlockInfos` from `WebSocketMessagesEncodable`...");
        let j = Json::try_from(m)?;
        Ok(serde_json::from_value(j)?)
    }
}
