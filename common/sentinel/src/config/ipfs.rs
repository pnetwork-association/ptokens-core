use derive_getters::Getters;
use serde::Deserialize;

use crate::{
    constants::{MAX_FREQUENCY, MIN_FREQUENCY},
    SentinelError,
};

#[derive(Debug, Default, Clone, Getters, Deserialize)]
pub struct IpfsConfig {
    ipfs_bin_path: String,
    status_update_frequency: u64,
}

impl IpfsConfig {
    pub fn new(ipfs_bin_path: String, status_update_frequency: u64) -> Result<Self, SentinelError> {
        if (MIN_FREQUENCY..=MAX_FREQUENCY).contains(&status_update_frequency) {
            Ok(Self {
                ipfs_bin_path,
                status_update_frequency,
            })
        } else {
            Err(SentinelError::InvalidFrequency {
                frequency: status_update_frequency,
                min: MIN_FREQUENCY,
                max: MAX_FREQUENCY,
            })
        }
    }
}
