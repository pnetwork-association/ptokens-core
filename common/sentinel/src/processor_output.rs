use std::fmt;

use common::BridgeSide;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{get_utc_timestamp, SentinelError, UserOps};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProcessorOutput {
    timestamp: u64,
    side: BridgeSide,
    latest_block_num: u64,
    processed_user_ops: UserOps,
}

impl ProcessorOutput {
    pub fn new(side: BridgeSide, latest_block_num: u64, processed_user_ops: UserOps) -> Result<Self, SentinelError> {
        Ok(Self {
            side,
            latest_block_num,
            processed_user_ops,
            timestamp: get_utc_timestamp()?,
        })
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn processed_use_ops(&self) -> UserOps {
        self.processed_user_ops.clone()
    }

    pub fn latest_block_num(&self) -> u64 {
        self.latest_block_num
    }

    pub fn side(&self) -> BridgeSide {
        self.side
    }

    pub fn has_user_ops(&self) -> bool {
        !self.processed_user_ops.is_empty()
    }
}

impl TryFrom<Json> for ProcessorOutput {
    type Error = SentinelError;

    fn try_from(j: Json) -> Result<Self, SentinelError> {
        Ok(serde_json::from_value(j)?)
    }
}

impl fmt::Display for ProcessorOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error pretty printing `ProcessorOutput` json: {e}"),
        }
    }
}
