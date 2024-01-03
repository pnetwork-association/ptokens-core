use std::{fmt, str::FromStr};

use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

use crate::{get_utc_timestamp, NetworkId, SentinelError, WebSocketMessagesEncodable};

#[derive(Debug, Error)]
pub enum DeltaError {
    #[error("not enough values to parse delta, got {0}, needed 5")]
    NotEnoughValues(usize),
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct Delta(u64);

impl fmt::Display for Delta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let delta = self.0;
        let s = delta % 60;
        let m = delta / 60 % 60;
        let h = delta / 60 / 60 % 24;
        let d = delta / 60 / 60 / 24 % 7;
        let w = delta / 60 / 60 / 24 / 7 % 52;
        write!(f, "{w}w:{d}d:{h}h:{m:0>2}m:{s:0>2}s")
    }
}

impl FromStr for Delta {
    type Err = DeltaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let xs = s
            .split(':')
            .map(|x| x[..x.len() - 1].parse::<u64>().unwrap_or_default())
            .collect::<Vec<u64>>();
        let n = xs.len();
        if n != 5 {
            Err(DeltaError::NotEnoughValues(n))
        } else {
            let r = (xs[0] * 7 * 24 * 60 * 60) + (xs[1] * 24 * 60 * 60) + (xs[2] * 60 * 60) + (xs[3] * 60) + xs[4];
            Ok(Self::new(r))
        }
    }
}

#[derive(Clone, Debug, Default, Deref, Constructor, Serialize, Deserialize)]
pub struct LatestBlockInfos(Vec<LatestBlockInfo>);

#[serde_as]
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize, Getters)]
pub struct LatestBlockInfo {
    // NOTE: Fields are public only to the crate so tests can modify them on the fly
    pub(crate) block_number: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) delta_from_now: Delta,
    pub(crate) block_timestamp: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) network_id: NetworkId,
}

impl LatestBlockInfo {
    pub fn new(block_number: u64, block_timestamp: u64, network_id: NetworkId) -> Self {
        let now = get_utc_timestamp().unwrap_or_default();
        let delta_from_now = Delta::new(if now > block_timestamp {
            now - block_timestamp
        } else {
            warn!("latest block timestamp appears to be ahead of now - defaulting to zero......");
            0
        });

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
        let t = get_utc_timestamp().unwrap() - 1337;
        let binance_str = "binance";
        let nid = NetworkId::from_str(binance_str).unwrap();
        let l = LatestBlockInfo::new(n, t, nid);
        let j = serde_json::json!(l);
        assert!(j.to_string().contains(binance_str)); // NOTE: We're testing the derived `DisplayFromStr` thing
        let r: LatestBlockInfo = serde_json::from_value(j).unwrap();
        assert_eq!(r, l);
    }

    #[test]
    fn should_format_delta_to_human_readable_timestamp_correctly() {
        let delta = Delta::new(3578941);
        let result = delta.to_string();
        let expected_result = "5w:6d:10h:09m:01s";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_make_delta_str_round_trip_correctly() {
        let d = Delta::new(13371337);
        let s = d.to_string();
        let r = Delta::from_str(&s).unwrap();
        assert_eq!(r, d);
    }
}
