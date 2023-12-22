use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_with::{serde_as, DisplayFromStr};

use crate::{get_utc_timestamp, NetworkId, SentinelError, WebSocketMessagesEncodable};

#[derive(Clone, Debug, Default, Deref, Constructor, Serialize, Deserialize)]
pub struct LatestBlockInfos(Vec<LatestBlockInfo>);

#[serde_as]
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize, Getters)]
pub struct LatestBlockInfo {
    // NOTE: Fields are public only to the crate so tests can modify them on the fly
    pub(crate) block_number: u64,
    pub(crate) delta_from_now: u64,
    pub(crate) block_timestamp: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) network_id: NetworkId,
}

impl LatestBlockInfo {
    pub fn new(block_number: u64, block_timestamp: u64, network_id: NetworkId) -> Self {
        let now = get_utc_timestamp().unwrap_or_default();
        let delta_from_now = if now > block_timestamp {
            now - block_timestamp
        } else {
            warn!("latest block timestamp appears to be ahead of now - defaulting to zero......");
            0
        };

        Self {
            network_id,
            block_number,
            delta_from_now,
            block_timestamp,
        }
    }
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn should_make_serde_json_roundtrip_correctly() {
        let n = 5;
        let t = get_utc_timestamp().unwrap();
        let binance_str = "binance";
        let nid = NetworkId::from_str(binance_str).unwrap();
        let l = LatestBlockInfo::new(n, t, nid);
        let j = serde_json::json!(l);
        assert!(j.to_string().contains(binance_str)); // NOTE: We're testing the derived `DisplayFromStr` thing
        let r: LatestBlockInfo = serde_json::from_value(j).unwrap();
        assert_eq!(r, l);
    }
}
