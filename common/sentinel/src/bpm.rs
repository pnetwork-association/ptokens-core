use std::{convert::From, fmt};

use bounded_vec_deque::BoundedVecDeque;
use derive_more::Deref;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::{NetworkId, ProcessorOutput};

type Timestamp = u64;
type LatestBlockNum = u64;
const MAX_SIZE: usize = 5;

#[derive(Debug, Clone, Deref)]
pub struct Bpms(Vec<Bpm>);

#[derive(Debug, Clone)]
pub struct Bpm(NetworkId, BoundedVecDeque<BpmInfo>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmInfo(Timestamp, LatestBlockNum);

impl BpmInfo {
    fn timestamp(&self) -> u64 {
        self.0
    }

    fn latest_block_num(&self) -> u64 {
        self.1
    }
}

impl From<&ProcessorOutput> for BpmInfo {
    fn from(v: &ProcessorOutput) -> Self {
        Self(v.timestamp(), v.latest_block_num())
    }
}

impl Default for Bpm {
    fn default() -> Self {
        Self::new(NetworkId::default())
    }
}

impl Bpm {
    pub fn network_id(&self) -> NetworkId {
        self.0
    }

    pub fn new(cid: NetworkId) -> Self {
        Self(cid, BoundedVecDeque::new(MAX_SIZE))
    }

    fn last_timestamp(&self) -> u64 {
        if self.1.is_empty() {
            0
        } else {
            self.1[self.1.len() - 1].timestamp()
        }
    }

    pub fn push(&mut self, o: &ProcessorOutput) {
        let last_timestamp = self.last_timestamp();
        let this_timestamp = o.timestamp();
        if this_timestamp > last_timestamp {
            self.1.push_back(BpmInfo::from(o));
        }
    }

    fn calc_bpm(&self) -> f64 {
        if self.1.len() < 2 {
            0.0
        } else {
            let a = self
                .1
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    if i == 0 {
                        None
                    } else {
                        let time_delta = (e.timestamp() - self.1[i - 1].timestamp()) as f64;
                        let block_delta = (e.latest_block_num() - self.1[i - 1].latest_block_num()) as f64;
                        let bpm: f64 = block_delta / (time_delta / 60.0);
                        Some(bpm)
                    }
                })
                .collect::<Vec<f64>>();
            a.iter().sum::<f64>() / a.len() as f64
        }
    }

    fn calc_bpm_string(&self) -> String {
        let r = self.calc_bpm();
        if r == 0.0 {
            "not enough data!".to_string()
        } else {
            format!("{r:.2}")
        }
    }

    fn to_json(&self) -> Json {
        json!({
            "cid": self.0,
            "bpm": self.calc_bpm_string(),
        })
    }
}

impl fmt::Display for Bpm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_json())
    }
}
