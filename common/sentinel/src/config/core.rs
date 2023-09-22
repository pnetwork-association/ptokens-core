use std::result::Result;

use derive_getters::Getters;
use serde::Deserialize;

use crate::{constants::MILLISECONDS_MULTIPLIER, SentinelError};

#[derive(Debug, Clone, Deserialize)]
pub struct SentinelCoreToml {
    timeout: u64,
    max_cancellable_time_delta: u64,
}

#[derive(Debug, Default, Clone, Getters)]
pub struct SentinelCoreConfig {
    timeout: u64,
    max_cancellable_time_delta: u64,
}

impl SentinelCoreConfig {
    pub fn from_toml(toml: &SentinelCoreToml) -> Result<Self, SentinelError> {
        Ok(Self {
            timeout: toml.timeout * MILLISECONDS_MULTIPLIER, // TODO sanity check?
            max_cancellable_time_delta: toml.max_cancellable_time_delta,
        })
    }
}
