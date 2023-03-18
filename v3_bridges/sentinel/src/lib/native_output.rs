use std::{fmt, result::Result};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{get_utc_timestamp, SentinelError};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NativeOutput {
    timestamp: u64,
    latest_block_num: u64,
}

impl NativeOutput {
    pub fn new(latest_block_num: u64) -> Result<Self, SentinelError> {
        Ok(Self {
            latest_block_num,
            timestamp: get_utc_timestamp()?,
        })
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn get_latest_block_num(&self) -> u64 {
        self.latest_block_num
    }
}

impl fmt::Display for NativeOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match serde_json::to_string(self) {
            Ok(s) => s,
            _ => "could not convert `NativeOutput` to json".to_string(),
        };
        write!(f, "{s}")
    }
}
