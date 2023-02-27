use std::str::FromStr;

use anyhow::Result;
use common_metadata::MetadataChainId;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::errors::SentinelError;

#[derive(Debug, Clone, Deserialize)]
struct EndpointsToml {
    host: Vec<String>,
    native: Vec<String>,
    host_chain_id: String,
    native_chain_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LogToml {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigToml {
    pub log: LogToml,
    pub batching: BatchingToml,
    pub endpoints: EndpointsToml,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchingToml {
    pub host_batch_size: usize,
    pub native_batch_size: usize,
    pub host_batch_duration: usize,
    pub native_batch_duration: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Batching {
    pub host_batch_size: usize,
    pub native_batch_size: usize,
    pub host_batch_duration: usize,
    pub native_batch_duration: usize,
}

impl Default for BatchingToml {
    fn default() -> Self {
        Self {
            host_batch_size: 1,
            native_batch_size: 1,
            host_batch_duration: 0,
            native_batch_duration: 0,
        }
    }
}

impl Batching {
    pub fn from_toml(toml: &BatchingToml) -> std::result::Result<Self, SentinelError> {
        Ok(Self {
            host_batch_size: Self::sanity_check_batch_size(toml.host_batch_size)?,
            native_batch_size: Self::sanity_check_batch_size(toml.native_batch_size)?,
            host_batch_duration: Self::sanity_check_batch_duration(toml.host_batch_duration)?,
            native_batch_duration: Self::sanity_check_batch_duration(toml.native_batch_duration)?,
        })
    }

    fn sanity_check_batch_size(batch_size: usize) -> std::result::Result<usize, SentinelError> {
        info!("Sanity checking batch size...");
        if batch_size > 0 && batch_size <= 1000 {
            Ok(batch_size)
        } else {
            Err(SentinelError::ConfigError(format!(
                "Batch size of {batch_size} is unacceptable"
            )))
        }
    }

    fn sanity_check_batch_duration(batch_duration: usize) -> std::result::Result<usize, SentinelError> {
        info!("Sanity checking batch duration...");
        // NOTE: A batch duration of 0 means we submit material one at a time...
        if batch_duration <= 60 * 10 {
            Ok(batch_duration)
        } else {
            Err(SentinelError::ConfigError(format!(
                "Batch duration of {batch_duration} is unacceptable"
            )))
        }
    }
}

const CONFIG_FILE_PATH: &str = "Config";

impl ConfigToml {
    pub fn new() -> Result<Self> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(CONFIG_FILE_PATH))
            .build()?
            .try_deserialize()?)
    }
}

pub struct Endpoints {
    host: Vec<String>,
    native: Vec<String>,
    host_chain_id: MetadataChainId,
    native_chain_id: MetadataChainId,
}

impl Endpoints {
    fn from_toml(toml: &EndpointsToml) -> Self {
        Self {
            host: toml.host.clone(),
            native: toml.native.clone(),
            host_chain_id: match MetadataChainId::from_str(&toml.host_chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `host_chain_id` from config, defaulting to `EthereumMainnet`");
                    warn!("{e}");
                    MetadataChainId::EthereumMainnet
                },
            },
            native_chain_id: match MetadataChainId::from_str(&toml.native_chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `native_chain_id` from config, defaulting to `EthereumMainnet`");
                    warn!("{e}");
                    MetadataChainId::EthereumMainnet
                },
            },
        }
    }

    pub fn get_first_endpoint(&self, is_native: bool) -> Result<String> {
        let endpoint_type = if is_native { "native" } else { "host" };
        info!("Getting first {endpoint_type} endpoint...");
        let err = format!("No {endpoint_type} endpoints in config file!");
        if is_native {
            if self.native.is_empty() {
                Err(anyhow!(err))
            } else {
                Ok(self.native[0].clone())
            }
        } else if self.host.is_empty() {
            Err(anyhow!(err))
        } else {
            Ok(self.host[0].clone())
        }
    }
}

pub struct Log {
    pub level: LogLevel,
}

impl Log {
    fn from_toml(toml: &LogToml) -> Result<Self> {
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

pub struct Config {
    pub log: Log,
    pub batching: Batching,
    pub endpoints: Endpoints,
}

impl Config {
    pub fn new() -> Result<Self> {
        Self::from_toml(&ConfigToml::new()?)
    }

    fn from_toml(toml: &ConfigToml) -> Result<Self> {
        Ok(Self {
            log: Log::from_toml(&toml.log)?,
            batching: Batching::from_toml(&toml.batching)?,
            endpoints: Endpoints::from_toml(&toml.endpoints),
        })
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_config() {
        let result = Config::new();
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_batch_size_sanity_check() {
        let mut toml = BatchingToml::default();
        let batch_size = usize::MAX;
        toml.host_batch_size = batch_size;
        let expected_error = format!("Batch size of {batch_size} is unacceptable");
        match Batching::from_toml(&toml) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(SentinelError::ConfigError(error)) => assert_eq!(error, expected_error),
            Err(error) => panic!("Wrong error received: {error}!"),
        }
    }

    #[test]
    fn should_fail_batch_duration_sanity_check() {
        let mut toml = BatchingToml::default();
        let duration = usize::MAX;
        toml.host_batch_duration = duration;
        let expected_error = format!("Batch duration of {duration} is unacceptable");
        match Batching::from_toml(&toml) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(SentinelError::ConfigError(error)) => assert_eq!(error, expected_error),
            Err(error) => panic!("Wrong error received: {error}!"),
        }
    }
}
