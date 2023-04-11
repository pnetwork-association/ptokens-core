use std::result::Result;

use common::BridgeSide;
use ethereum_types::Address as EthAddress;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::{
    config::{
        BatchingConfig,
        BatchingToml,
        ConfigT,
        CoreConfig,
        CoreToml,
        HostConfig,
        HostToml,
        LogConfig,
        LogToml,
        MongoConfig,
        MongoToml,
        NativeConfig,
        NativeToml,
    },
    Endpoints,
    SentinelError,
};

const CONFIG_FILE_PATH: &str = "config";

#[derive(Debug, Clone, Deserialize)]
struct ConfigToml {
    log: LogToml,
    host: HostToml,
    core: CoreToml,
    mongo: MongoToml,
    native: NativeToml,
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
    log: LogConfig,
    host: HostConfig,
    core: CoreConfig,
    mongo: MongoConfig,
    native: NativeConfig,
    batching: BatchingConfig,
}

impl Config {
    pub fn mongo(&self) -> &MongoConfig {
        &self.mongo
    }

    pub fn host(&self) -> &HostConfig {
        &self.host
    }

    pub fn native(&self) -> &NativeConfig {
        &self.native
    }

    pub fn log(&self) -> &LogConfig {
        &self.log
    }

    pub fn core(&self) -> &CoreConfig {
        &self.core
    }

    pub fn batching(&self) -> &BatchingConfig {
        &self.batching
    }

    pub fn get_db_path(&self) -> String {
        self.core().get_db_path()
    }

    pub fn new() -> Result<Self, SentinelError> {
        let res = Self::from_toml(&ConfigToml::new()?)?;
        debug!("Config {:?}", res);
        Ok(res)
    }

    fn from_toml(toml: &ConfigToml) -> Result<Self, SentinelError> {
        Ok(Self {
            log: LogConfig::from_toml(&toml.log)?,
            core: CoreConfig::from_toml(&toml.core)?,
            host: HostConfig::from_toml(&toml.host)?,
            mongo: MongoConfig::from_toml(&toml.mongo),
            native: NativeConfig::from_toml(&toml.native)?,
            batching: BatchingConfig::from_toml(&toml.batching)?,
        })
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log.level
    }

    pub fn get_host_endpoints(&self) -> Endpoints {
        self.host.get_endpoints()
    }

    pub fn get_native_endpoints(&self) -> Endpoints {
        self.native.get_endpoints()
    }

    pub fn state_manager(&self, side: &BridgeSide) -> EthAddress {
        if side.is_native() {
            self.native.state_manager()
        } else {
            self.host.state_manager()
        }
    }

    pub fn router(&self, side: &BridgeSide) -> EthAddress {
        if side.is_native() {
            self.native.router()
        } else {
            self.host.router()
        }
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
