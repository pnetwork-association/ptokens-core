use std::result::Result;

use log::Level as LogLevel;
use serde::Deserialize;

use super::ConfigError;
use crate::SentinelError;

#[derive(Debug, Clone, Deserialize)]
pub struct LogToml {
    pub path: String,
    pub level: String,
    pub enabled: bool,
    pub max_log_size: u64,
    pub max_num_logs: usize,
    pub use_file_logging: bool,
}

#[derive(Debug, Clone)]
pub struct LogConfig {
    enabled: bool,
    pub path: String,
    pub level: LogLevel,
    pub max_log_size: u64,
    pub max_num_logs: usize,
    pub use_file_logging: bool,
}

impl LogConfig {
    pub fn from_toml(toml: &LogToml) -> Result<Self, SentinelError> {
        Ok(Self {
            enabled: toml.enabled,
            path: toml.path.clone(),
            use_file_logging: toml.use_file_logging,
            max_num_logs: Self::sanity_check_max_num_logs(toml.max_num_logs)?,
            max_log_size: Self::sanity_check_max_log_size(toml.max_log_size)?,
            level: match toml.level.to_lowercase().as_str() {
                "warn" => LogLevel::Warn,
                "debug" => LogLevel::Debug,
                "trace" => LogLevel::Trace,
                _ => LogLevel::Info,
            },
        })
    }

    fn sanity_check_max_num_logs(n: usize) -> Result<usize, SentinelError> {
        const MIN: usize = 1;
        const MAX: usize = 1_000_000;
        if (MIN..=MAX).contains(&n) {
            Ok(n)
        } else {
            Err(SentinelError::SentinelConfig(ConfigError::LogNum {
                size: n,
                max: MAX,
                min: MIN,
            }))
        }
    }

    fn sanity_check_max_log_size(n: u64) -> Result<u64, SentinelError> {
        const MIN: u64 = 1_000_000; // NOTE: 1mb
        const MAX: u64 = 1_000_000_000_000; // NOTE: 1 tb
        if (MIN..=MAX).contains(&n) {
            Ok(n)
        } else {
            Err(SentinelError::SentinelConfig(ConfigError::LogSize {
                size: n,
                max: MAX,
                min: MIN,
            }))
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
