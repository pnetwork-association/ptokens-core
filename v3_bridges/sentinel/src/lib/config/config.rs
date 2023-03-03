use std::str::FromStr;

use anyhow::Result;
use common_metadata::MetadataChainId;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::config::{BatchingConfig, BatchingToml, HostConfig, HostToml, LogConfig, LogToml, NativeConfig, NativeToml};

const CONFIG_FILE_PATH: &str = "Config";

#[derive(Debug, Clone, Deserialize)]
struct ConfigToml {
    pub log: LogToml,
    pub host: HostToml,
    pub native: NativeToml,
    pub batching: BatchingToml,
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
    pub host: HostConfig,
    pub native: NativeConfig,
    pub batching: BatchingConfig,
}

impl Config {
    pub fn new() -> Result<Self> {
        let res = Self::from_toml(&ConfigToml::new()?)?;
        debug!("Config {:?}", res);
        Ok(res)
    }

    fn from_toml(toml: &ConfigToml) -> Result<Self> {
        Ok(Self {
            log: LogConfig::from_toml(&toml.log)?,
            host: HostConfig::from_toml(&toml.host),
            native: NativeConfig::from_toml(&toml.native),
            batching: BatchingConfig::from_toml(&toml.batching)?,
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
