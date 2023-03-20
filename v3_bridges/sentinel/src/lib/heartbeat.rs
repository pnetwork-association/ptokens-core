use std::{convert::From, fmt};

use bounded_vec_deque::BoundedVecDeque;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{HostOutput, NativeOutput};

type Timestamp = u64;
type LatestBlockNum = u64;
const MAX_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct Heartbeats {
    host_deque: BoundedVecDeque<HeartbeatInfo>,
    native_deque: BoundedVecDeque<HeartbeatInfo>,
}

impl Default for Heartbeats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatsJson {
    #[serde(rename = "_id")]
    id: String,
    host: Vec<HeartbeatInfo>,
    native: Vec<HeartbeatInfo>,
}

impl Default for HeartbeatsJson {
    fn default() -> Self {
        Self {
            host: vec![],
            native: vec![],
            id: "heartbeats".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatInfo((Timestamp, LatestBlockNum));

impl HeartbeatInfo {
    fn get_timestamp(&self) -> u64 {
        self.0 .0
    }

    fn get_latest_block_num(&self) -> u64 {
        self.0 .1
    }
}

impl From<&NativeOutput> for HeartbeatInfo {
    fn from(v: &NativeOutput) -> Self {
        Self((v.get_timestamp(), v.get_latest_block_num()))
    }
}

impl From<&HostOutput> for HeartbeatInfo {
    fn from(v: &HostOutput) -> Self {
        Self((v.get_timestamp(), v.get_latest_block_num()))
    }
}

impl Heartbeats {
    pub fn to_json(&self) -> HeartbeatsJson {
        HeartbeatsJson {
            id: "heartbeats".to_string(),
            host: self.host_deque.iter().cloned().collect::<Vec<_>>(),
            native: self.native_deque.iter().cloned().collect::<Vec<_>>(),
        }
    }

    pub fn from_json(json: &HeartbeatsJson) -> Self {
        Self {
            host_deque: BoundedVecDeque::from_iter(json.host.iter().cloned(), MAX_SIZE),
            native_deque: BoundedVecDeque::from_iter(json.native.iter().cloned(), MAX_SIZE),
        }
    }

    pub fn new() -> Self {
        Self {
            host_deque: BoundedVecDeque::new(MAX_SIZE),
            native_deque: BoundedVecDeque::new(MAX_SIZE),
        }
    }

    fn get_last_native_timestamp(&self) -> u64 {
        if self.native_deque.is_empty() {
            0
        } else {
            self.native_deque[self.native_deque.len() - 1].get_timestamp()
        }
    }

    fn get_last_host_timestamp(&self) -> u64 {
        if self.host_deque.is_empty() {
            0
        } else {
            self.host_deque[self.host_deque.len() - 1].get_timestamp()
        }
    }

    pub fn push_native(&mut self, o: &NativeOutput) {
        let last_timestamp = self.get_last_native_timestamp();
        let this_timestamp = o.get_timestamp();

        if this_timestamp > last_timestamp {
            self.native_deque.push_back(HeartbeatInfo::from(o));
        }
    }

    pub fn push_host(&mut self, o: &HostOutput) {
        let last_timestamp = self.get_last_host_timestamp();
        let this_timestamp = o.get_timestamp();
        if this_timestamp > last_timestamp {
            self.host_deque.push_back(HeartbeatInfo::from(o));
        }
    }

    fn calc_bpm(deque: &BoundedVecDeque<HeartbeatInfo>) -> f64 {
        if deque.len() < 2 {
            0.0
        } else {
            let a = deque
                .iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    if i == 0 {
                        None
                    } else {
                        let time_delta = (e.get_timestamp() - deque[i - 1].get_timestamp()) as f64;
                        let block_delta = (e.get_latest_block_num() - deque[i - 1].get_latest_block_num()) as f64;
                        let bpm: f64 = block_delta / (time_delta / 60.0);
                        Some(bpm)
                    }
                })
                .collect::<Vec<f64>>();
            a.iter().sum::<f64>() / a.len() as f64
        }
    }

    fn calc_bpm_string(deque: &BoundedVecDeque<HeartbeatInfo>) -> String {
        let r = Self::calc_bpm(deque);
        if r == 0.0 {
            "not enough data!".to_string()
        } else {
            format!("{r:.2}")
        }
    }

    fn get_host_heartbeat(&self) -> String {
        Self::calc_bpm_string(self.get_host_deque())
    }

    fn get_native_heartbeat(&self) -> String {
        Self::calc_bpm_string(self.get_native_deque())
    }

    fn get_host_deque(&self) -> &BoundedVecDeque<HeartbeatInfo> {
        &self.host_deque
    }

    fn get_native_deque(&self) -> &BoundedVecDeque<HeartbeatInfo> {
        &self.native_deque
    }

    pub fn to_output(&self) -> HeartbeatsOutput {
        HeartbeatsOutput {
            host_bpm: self.get_host_heartbeat(),
            native_bpm: self.get_native_heartbeat(),
            host_data: self.host_deque.iter().cloned().collect::<Vec<_>>(),
            native_data: self.native_deque.iter().cloned().collect::<Vec<_>>(),
        }
    }
}

impl HeartbeatsJson {
    pub fn to_output(&self) -> HeartbeatsOutput {
        Heartbeats::from_json(self).to_output()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct HeartbeatsOutput {
    host_bpm: String,
    native_bpm: String,
    host_data: Vec<HeartbeatInfo>,
    native_data: Vec<HeartbeatInfo>,
}

impl fmt::Display for Heartbeats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let h = self.get_host_heartbeat();
        let n = self.get_native_heartbeat();
        let j = json!({ "native_bpm": format!("{n}"), "host_bpm": format!("{h}")});
        write!(f, "{j}")
    }
}

impl fmt::Display for HeartbeatsJson {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "error converting `HeartbeatsJson` to string: {e}"),
        }
    }
}