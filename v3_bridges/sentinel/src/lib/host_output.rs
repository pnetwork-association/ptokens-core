use std::{fmt, result::Result};

use serde::{Deserialize, Serialize};

use crate::{get_utc_timestamp, SentinelError};

// TODO use serde to add prefixex?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostOutput {
    timestamp: u64,
    latest_block_num: u64,
}

impl HostOutput {
    pub fn new(latest_block_num: u64) -> Result<Self, SentinelError> {
        Ok(Self {
            latest_block_num,
            timestamp: get_utc_timestamp()?,
        })
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
