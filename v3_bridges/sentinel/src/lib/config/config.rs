use std::str::FromStr;

use anyhow::Result;
use common_metadata::MetadataChainId;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::config::{BatchingConfig, BatchingToml, EndpointsConfig, EndpointsToml, LogConfig, LogToml};

const CONFIG_FILE_PATH: &str = "Config";

#[derive(Debug, Clone, Deserialize)]
struct ConfigToml {
    pub log: LogToml,
    pub batching: BatchingToml,
    pub endpoints: EndpointsToml,
}

impl ConfigToml {
    pub fn new() -> Result<Self> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(CONFIG_FILE_PATH))
            .build()?
            .try_deserialize()?)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub log: LogConfig,
    pub batching: BatchingConfig,
    pub endpoints: EndpointsConfig,
}

impl Config {
    pub fn new() -> Result<Self> {
        let res = Self::from_toml(&ConfigToml::new()?)?;
        debug!("res {:?}", res);
        Ok(res)
    }

    fn from_toml(toml: &ConfigToml) -> Result<Self> {
        Ok(Self {
            log: LogConfig::from_toml(&toml.log)?,
            batching: BatchingConfig::from_toml(&toml.batching)?,
            endpoints: EndpointsConfig::from_toml(&toml.endpoints),
        })
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log.level
    }
}

mod tests {
    use super::*;

    #[test]
    fn should_get_config() {
        let result = Config::new();
        assert!(result.is_ok());
    }
}
