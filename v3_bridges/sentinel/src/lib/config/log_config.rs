use std::str::FromStr;

use anyhow::Result;
use common_metadata::MetadataChainId;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::errors::SentinelError;

#[derive(Debug, Clone, Deserialize)]
pub struct LogToml {
    pub level: String,
}

#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: LogLevel,
}

impl LogConfig {
    pub fn from_toml(toml: &LogToml) -> Result<Self> {
        Ok(Self {
            level: match toml.level.to_lowercase().as_str() {
                "warn" => LogLevel::Warn,
                "debug" => LogLevel::Debug,
                "trace" => LogLevel::Trace,
                _ => LogLevel::Info,
            },
        })
    }
}
