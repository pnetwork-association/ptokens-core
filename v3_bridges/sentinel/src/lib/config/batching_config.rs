use serde::Deserialize;

use crate::errors::SentinelError;

#[derive(Debug, Clone, Deserialize)]
pub struct BatchingToml {
    pub host_batch_size: u64,
    pub native_batch_size: u64,
    pub host_batch_duration: u64,
    pub native_batch_duration: u64,
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
    pub fn from_toml(toml: &BatchingToml) -> std::result::Result<Self, SentinelError> {
        Ok(Self {
            host_batch_size: Self::sanity_check_batch_size(toml.host_batch_size)?,
            native_batch_size: Self::sanity_check_batch_size(toml.native_batch_size)?,
            host_batch_duration: Self::sanity_check_batch_duration(toml.host_batch_duration)?,
            native_batch_duration: Self::sanity_check_batch_duration(toml.native_batch_duration)?,
        })
    }

    fn sanity_check_batch_size(batch_size: u64) -> std::result::Result<u64, SentinelError> {
        info!("Sanity checking batch size...");
        if batch_size > 0 && batch_size <= 1000 {
            Ok(batch_size)
        } else {
            Err(SentinelError::ConfigError(format!(
                "Batch size of {batch_size} is unacceptable"
            )))
        }
    }

    fn sanity_check_batch_duration(batch_duration: u64) -> std::result::Result<u64, SentinelError> {
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
        let expected_error = format!("Batch size of {batch_size} is unacceptable");
        match BatchingConfig::from_toml(&toml) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(SentinelError::ConfigError(error)) => assert_eq!(error, expected_error),
            Err(error) => panic!("Wrong error received: {error}!"),
        }
    }

    #[test]
    fn should_fail_batch_duration_sanity_check() {
        let mut toml = BatchingToml::default();
        let duration = u64::MAX;
        toml.host_batch_duration = duration;
        let expected_error = format!("Batch duration of {duration} is unacceptable");
        match BatchingConfig::from_toml(&toml) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(SentinelError::ConfigError(error)) => assert_eq!(error, expected_error),
            Err(error) => panic!("Wrong error received: {error}!"),
        }
    }
}
