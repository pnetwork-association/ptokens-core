use std::{path::Path, result::Result, str::FromStr};

use common::{CoreType, V3CoreType};
use derive_getters::Getters;
use serde::Deserialize;

use crate::{constants::MILLISECONDS_MULTIPLIER, SentinelError};

#[derive(Debug, Clone, Deserialize)]
pub struct SentinelCoreToml {
    timeout: u64,
    db_path: String,
    core_type: String,
    max_cancellable_time_delta: u64,
}

#[derive(Debug, Default, Clone, Getters)]
pub struct SentinelCoreConfig {
    timeout: u64,
    db_path: String,

    #[getter(skip)]
    core_type: CoreType,

    #[getter(skip)]
    max_cancellable_time_delta: u64,
}

impl SentinelCoreConfig {
    pub fn core_type(&self) -> CoreType {
        self.core_type
    }

    pub fn max_cancellable_time_delta(&self) -> u64 {
        self.max_cancellable_time_delta
    }

    pub fn from_toml(toml: &SentinelCoreToml) -> Result<Self, SentinelError> {
        Ok(Self {
            db_path: toml.db_path.clone(),
            timeout: toml.timeout * MILLISECONDS_MULTIPLIER, // TODO sanity check?
            max_cancellable_time_delta: toml.max_cancellable_time_delta,
            core_type: CoreType::V3(V3CoreType::from_str(&toml.core_type)?),
        })
    }

    pub fn db_exists(&self) -> bool {
        Path::new(&self.db_path).exists()
    }

    pub fn get_db_path(&self) -> String {
        self.db_path.clone()
    }
}
