use std::{path::Path, result::Result};

use serde::Deserialize;

const CONFIG_FILE_PATH: &str = "./src/bin/core/core-config";

use lib::{LogConfig, LogToml};

use crate::error::CoreError;

#[derive(Debug, Clone, Deserialize)]
pub struct CoreConfigToml {
    db_path: String,
    log: LogToml,
}

impl CoreConfigToml {
    pub fn new() -> Result<Self, CoreError> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(CONFIG_FILE_PATH))
            .build()?
            .try_deserialize()?)
    }
}

#[derive(Debug, Clone)]
pub struct CoreConfig {
    db_path: String,
    log: LogConfig,
}

impl CoreConfig {
    pub fn log(&self) -> LogConfig {
        self.log.clone()
    }

    pub fn db_path(&self) -> String {
        self.db_path.clone()
    }

    pub fn from_toml(toml: &CoreConfigToml) -> Result<Self, CoreError> {
        Ok(Self {
            db_path: toml.db_path.clone(),
            log: LogConfig::from_toml(&toml.log)?,
        })
    }

    pub fn db_exists(&self) -> bool {
        Path::new(&self.db_path).exists()
    }

    fn check_db_exists(self) -> Result<Self, CoreError> {
        if self.db_exists() {
            Ok(self)
        } else {
            Err(CoreError::NoDb(self.db_path()))
        }
    }

    pub fn new() -> Result<Self, CoreError> {
        Self::from_toml(&CoreConfigToml::new()?).and_then(Self::check_db_exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_config() {
        let result = CoreConfig::new();
        result.unwrap();
    }
}
