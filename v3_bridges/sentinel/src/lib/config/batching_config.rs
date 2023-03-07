use std::result::Result;

use serde::Deserialize;

use crate::{config::Error, SentinelError};

#[derive(Debug, Clone, Deserialize)]
pub struct BatchingToml {
    host_batch_size: u64,
    native_batch_size: u64,
    host_batch_duration: u64,
    native_batch_duration: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchingConfig {
    pub host_batch_size: u64,
    pub native_batch_size: u64,
    pub host_batch_duration: u64,
    pub native_batch_duration: u64,
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

impl BatchingConfig {
    pub fn from_toml(toml: &BatchingToml) -> Result<Self, SentinelError> {
        Ok(Self {
            host_batch_size: Self::sanity_check_batch_size(toml.host_batch_size)?,
            native_batch_size: Self::sanity_check_batch_size(toml.native_batch_size)?,
            host_batch_duration: Self::sanity_check_batch_duration(toml.host_batch_duration)?,
            native_batch_duration: Self::sanity_check_batch_duration(toml.native_batch_duration)?,
        })
    }

    fn sanity_check_batch_size(batch_size: u64) -> Result<u64, SentinelError> {
        info!("Sanity checking batch size...");
        const MIN: u64 = 0;
        const MAX: u64 = 1000;
        if batch_size > MIN && batch_size <= MAX {
            Ok(batch_size)
        } else {
            Err(SentinelError::ConfigError(Error::BatchSizeError {
                size: batch_size,
                min: MIN,
                max: MAX,
            }))
        }
    }

    fn sanity_check_batch_duration(batch_duration: u64) -> Result<u64, SentinelError> {
        info!("Sanity checking batch duration...");
        // NOTE: A batch duration of 0 means we submit material one at a time...
        const MAX: u64 = 60 * 10; // NOTE: Ten mins
        if batch_duration <= MAX {
            Ok(batch_duration)
        } else {
            Err(SentinelError::ConfigError(Error::BatchDurationError {
                max: MAX,
                size: batch_duration,
            }))
        }
    }

    pub fn get_batch_size(&self, is_native: bool) -> u64 {
        debug!(
            "Getting {} batch size from config",
            if is_native { "native" } else { "host" }
        );
        if is_native {
            self.native_batch_size
        } else {
            self.host_batch_size
        }
    }

    pub fn get_batch_duration(&self, is_native: bool) -> u64 {
        debug!(
            "Getting {} batch duration from config",
            if is_native { "native" } else { "host" }
        );
        if is_native {
            self.native_batch_duration
        } else {
            self.host_batch_duration
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_fail_batch_size_sanity_check() {
        let mut toml = BatchingToml::default();
        let batch_size = u64::MAX;
        toml.host_batch_size = batch_size;
        let expected_error = Error::BatchSizeError {
            size: batch_size,
            min: 0,
            max: 1000,
        };
        match BatchingConfig::from_toml(&toml) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(SentinelError::ConfigError(e)) => assert_eq!(e, expected_error),
            Err(error) => panic!("Wrong error received: {error}!"),
        }
    }

    #[test]
    fn should_fail_batch_duration_sanity_check() {
        let mut toml = BatchingToml::default();
        let duration = u64::MAX;
        toml.host_batch_duration = duration;
        let expected_error = Error::BatchDurationError {
            size: duration,
            max: 600,
        };
        match BatchingConfig::from_toml(&toml) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(SentinelError::ConfigError(e)) => assert_eq!(e, expected_error),
            Err(error) => panic!("Wrong error received: {error}!"),
        }
    }
}
