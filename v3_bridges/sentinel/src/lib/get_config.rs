use anyhow::Result;
use log::Level as LogLevel;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Endpoints {
    pub host: Vec<String>,
    pub native: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
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
