use std::{fmt, result::Result};

use serde::{Deserialize, Serialize};

use crate::{get_utc_timestamp, SentinelError, UserOperations};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostOutput {
    timestamp: u64,
    latest_block_num: u64,
    host_unmatched_user_ops: UserOperations,
    native_unmatched_user_ops: UserOperations,
}

impl HostOutput {
    pub fn new(latest_block_num: u64) -> Result<Self, SentinelError> {
        Ok(Self {
            latest_block_num,
            timestamp: get_utc_timestamp()?,
            host_unmatched_user_ops: UserOperations::empty(),
            native_unmatched_user_ops: UserOperations::empty(),
        })
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn get_latest_block_num(&self) -> u64 {
        self.latest_block_num
    }

    pub fn add_unmatched_user_ops(&mut self, n: &UserOperations, h: &UserOperations) {
        self.host_unmatched_user_ops = h.clone();
        self.native_unmatched_user_ops = n.clone();
    }
}

impl fmt::Display for HostOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match serde_json::to_string(self) {
            Ok(s) => s,
            _ => "could not convert `HostOutput` to json".to_string(),
        };
        write!(f, "{s}")
    }
}
