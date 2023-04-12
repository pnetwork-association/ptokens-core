use std::fmt;

use common::BridgeSide;
use serde::{Deserialize, Serialize};

use crate::{get_utc_timestamp, SentinelError, UserOps};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Output {
    timestamp: u64,
    side: BridgeSide,
    user_ops: UserOps,
    latest_block_num: u64,
}

impl Output {
    pub fn new(side: BridgeSide, latest_block_num: u64, user_ops: UserOps) -> Result<Self, SentinelError> {
        Ok(Self {
            side,
            user_ops,
            latest_block_num,
            timestamp: get_utc_timestamp()?,
        })
    }

    pub fn user_ops(&self) -> UserOps {
        self.user_ops.clone()
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn latest_block_num(&self) -> u64 {
        self.latest_block_num
    }

    pub fn side(&self) -> BridgeSide {
        self.side
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error pretty printing `Output` json: {e}"),
        }
    }
}
