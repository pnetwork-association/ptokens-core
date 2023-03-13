use std::result::Result;

use log::Level as LogLevel;
use serde::Deserialize;

use crate::{
    config::{
        BatchingConfig,
        BatchingToml,
        HostConfig,
        HostToml,
        LogConfig,
        LogToml,
        MongoConfig,
        NativeConfig,
        NativeToml,
    },
    SentinelError,
};

const CONFIG_FILE_PATH: &str = "config";

#[derive(Debug, Clone, Deserialize)]
struct ConfigToml {
    log: LogToml,
    host: HostToml,
    native: NativeToml,
    mongo: MongoConfig,
    batching: BatchingToml,
}

impl ConfigToml {
    pub fn new() -> Result<Self, SentinelError> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(CONFIG_FILE_PATH))
            .build()?
            .try_deserialize()?)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub log_config: LogConfig,
    pub host_config: HostConfig,
    pub mongo_config: MongoConfig,
    pub native_config: NativeConfig,
    pub batching_config: BatchingConfig,
}

impl Config {
    pub fn new() -> Result<Self, SentinelError> {
        let res = Self::from_toml(&ConfigToml::new()?)?;
        debug!("Config {:?}", res);
        Ok(res)
    }

    fn from_toml(toml: &ConfigToml) -> Result<Self, SentinelError> {
        Ok(Self {
            mongo_config: toml.mongo.clone(),
            log_config: LogConfig::from_toml(&toml.log)?,
            host_config: HostConfig::from_toml(&toml.host)?,
            native_config: NativeConfig::from_toml(&toml.native)?,
            batching_config: BatchingConfig::from_toml(&toml.batching)?,
        })
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log_config.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_config() {
        let result = Config::new();
        result.unwrap();
    }
}