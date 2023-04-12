use std::{fmt, result::Result};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{get_utc_timestamp, SentinelError, UserOps};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NativeOutput {
    timestamp: u64,
    user_ops: UserOps,
    latest_block_num: u64,
}

impl NativeOutput {
    pub fn new(latest_block_num: u64, user_ops: UserOps) -> Result<Self, SentinelError> {
        Ok(Self {
            user_ops,
            latest_block_num,
            timestamp: get_utc_timestamp()?,
        })
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn latest_block_num(&self) -> u64 {
        self.latest_block_num
    }

    pub fn user_ops(&self) -> UserOps {
        self.user_ops.clone()
    }
}

impl fmt::Display for NativeOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match serde_json::to_string_pretty(self) {
            Ok(s) => s,
            _ => "could not convert `NativeOutput` to json".to_string(),
        };
        write!(f, "{s}")
    }
}
