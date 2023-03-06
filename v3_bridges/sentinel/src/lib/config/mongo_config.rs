use std::str::FromStr;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MongoToml {
    uri: String,
    database: String,
    collection: String,
}

#[derive(Debug, Clone)]
pub struct MongoConfig {
    // TODO save & instantiate the connection here!
}

impl MongoConfig {
    pub fn from_toml(toml: &MongoToml) -> Result<Self> {
        Ok(Self {})
    }
}
