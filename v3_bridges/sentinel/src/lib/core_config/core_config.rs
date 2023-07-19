use std::{path::Path, result::Result};

use serde::Deserialize;

const CONFIG_FILE_PATH: &str = "core-config";

use crate::SentinelError;

#[derive(Debug, Clone, Deserialize)]
pub struct CoreConfigToml {
    db_path: String,
}

impl CoreConfigToml {
    pub fn new() -> Result<Self, SentinelError> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(CONFIG_FILE_PATH))
            .build()?
            .try_deserialize()?)
    }
}

#[derive(Debug, Clone)]
pub struct CoreConfig {
    db_path: String,
}

impl CoreConfig {
    pub fn db_path(&self) -> String {
        self.db_path.clone()
    }

    pub fn from_toml(toml: &CoreConfigToml) -> Result<Self, SentinelError> {
        Ok(Self {
            db_path: toml.db_path.clone(),
        })
    }

    pub fn db_exists(&self) -> bool {
        Path::new(&self.db_path).exists()
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
