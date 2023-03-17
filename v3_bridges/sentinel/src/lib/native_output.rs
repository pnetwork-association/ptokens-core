use std::result::Result;

use serde::{Deserialize, Serialize};

use crate::{get_utc_timestamp, SentinelError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeOutput {
    timestamp: u128,
    latest_block_num: u64,
}

impl NativeOutput {
    pub fn new(latest_block_num: u64) -> Result<Self, SentinelError> {
        Ok(Self {
            latest_block_num,
            timestamp: get_utc_timestamp()?,
        })
    }
}
