use std::result::Result;

use common::BridgeSide;
use ethereum_types::Address as EthAddress;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::{
    sentinel_config::{
        BatchingConfig,
        BatchingToml,
        ConfigT,
        HostConfig,
        HostToml,
        LogConfig,
        LogToml,
        MongoConfig,
        MongoToml,
        NativeConfig,
        NativeToml,
        SentinelCoreConfig,
        SentinelCoreToml,
    },
    Endpoints,
    SentinelError,
};

const CONFIG_FILE_PATH: &str = "sentinel-config";

#[derive(Debug, Clone, Deserialize)]
struct SentinelConfigToml {
    log: LogToml,
    host: HostToml,
    mongo: MongoToml,
    native: NativeToml,
    core: SentinelCoreToml,
    batching: BatchingToml,
}

impl SentinelConfigToml {
    pub fn new() -> Result<Self, SentinelError> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(CONFIG_FILE_PATH))
            .build()?
            .try_deserialize()?)
    }
}

#[derive(Debug, Clone)]
pub struct SentinelConfig {
    log: LogConfig,
    host: HostConfig,
    mongo: MongoConfig,
    native: NativeConfig,
    core: SentinelCoreConfig,
    batching: BatchingConfig,
}

impl SentinelConfig {
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

    pub fn core(&self) -> &SentinelCoreConfig {
        &self.core
    }

    pub fn batching(&self) -> &BatchingConfig {
        &self.batching
    }

    pub fn get_db_path(&self) -> String {
        self.core().get_db_path()
    }

    pub fn new() -> Result<Self, SentinelError> {
        let res = Self::from_toml(&SentinelConfigToml::new()?)?;
        debug!("sentinel config {:?}", res);
        Ok(res)
    }

    fn from_toml(toml: &SentinelConfigToml) -> Result<Self, SentinelError> {
        Ok(Self {
            log: LogConfig::from_toml(&toml.log)?,
            core: SentinelCoreConfig::from_toml(&toml.core)?,
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
        self.host.endpoints()
    }

    pub fn get_native_endpoints(&self) -> Endpoints {
        self.native.endpoints()
    }

    pub fn is_validating(&self, side: &BridgeSide) -> bool {
        if side.is_native() {
            self.native.is_validating()
        } else {
            self.host.is_validating()
        }
    }

    pub fn pnetwork_hub(&self, side: &BridgeSide) -> EthAddress {
        if side.is_native() {
            self.native.pnetwork_hub()
        } else {
            self.host.pnetwork_hub()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_config() {
        let result = SentinelConfig::new();
        result.unwrap();
    }
}