use anyhow::Result;
use log::Level as LogLevel;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Endpoints {
    host: Vec<String>,
    native: Vec<String>,
}

impl Endpoints {
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

#[derive(Debug, Clone, Deserialize)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub log: Log,
    pub endpoints: Endpoints,
}

const CONFIG_FILE_PATH: &str = "Config";

impl Config {
    pub fn new() -> Result<Self> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(CONFIG_FILE_PATH))
            .build()?
            .try_deserialize()?)
    }

    pub fn get_log_level(&self) -> LogLevel {
        match self.log.level.to_lowercase().as_str() {
            "warn" => LogLevel::Warn,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
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
}
