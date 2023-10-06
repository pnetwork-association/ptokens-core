use derive_getters::Getters;
use serde::Deserialize;

use crate::{SentinelError, SentinelStatusError, MAX_STATUS_PUBLISHING_FREQENCY, MIN_STATUS_PUBLISHING_FREQENCY};

#[derive(Debug, Default, Clone, Getters, Deserialize)]
pub struct IpfsConfig {
    ipfs_bin_path: String,
    status_update_frequency: u64,
}

impl IpfsConfig {
    pub fn new(ipfs_bin_path: String, status_update_frequency: u64) -> Result<Self, SentinelError> {
        if (MIN_STATUS_PUBLISHING_FREQENCY..=MAX_STATUS_PUBLISHING_FREQENCY).contains(&status_update_frequency) {
            Ok(Self {
                ipfs_bin_path,
                status_update_frequency,
            })
        } else {
            Err(SentinelStatusError::InvalidPublishingFrequency(status_update_frequency).into())
        }
    }
}
