use std::{result::Result, str::FromStr};

use common::{CoreType, V3CoreType};
use serde::Deserialize;

use crate::SentinelError;

#[derive(Debug, Clone, Deserialize)]
pub struct CoreToml {
    db_path: String,
    core_type: String,
}

#[derive(Debug, Default, Clone)]
pub struct CoreConfig {
    pub db_path: String,
    pub core_type: CoreType,
}

impl CoreConfig {
    pub fn from_toml(toml: &CoreToml) -> Result<Self, SentinelError> {
        Ok(Self {
            db_path: toml.db_path.clone(),
            core_type: CoreType::V3(V3CoreType::from_str(&toml.core_type)?),
        })
    }
}
