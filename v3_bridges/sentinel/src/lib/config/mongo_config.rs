use std::{result::Result, time::Duration};

use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client,
    Collection,
    Database,
};
use serde::Deserialize;

use crate::{HeartbeatsJson, HostOutput, NativeOutput, SentinelError};

#[derive(Debug, Clone, Deserialize)]
pub struct MongoToml {
    uri: String,
    timeout: u32,
    database: String,
    host_collection: String,
    native_collection: String,
}

#[derive(Debug, Clone)]
pub struct MongoConfig {
    uri: String,
    database: String,
    host_collection: String,
    timeout: Option<Duration>,
    native_collection: String,
}

impl MongoConfig {
    fn sanity_check_timeout(timeout: u32) -> Option<Duration> {
        let min = 1;
        let max = 60 * 2;
        let default = 10;
        let d = if (min..=max).contains(&timeout) {
            Duration::new(timeout.into(), 0)
        } else {
            warn!("timeout of {timeout} is not > {min} && < {max}, using default of {default}s!");
            Duration::new(default, 0)
        };
        Some(d)
    }

    pub async fn from_toml(toml: &MongoToml) -> Result<Self, SentinelError> {
        let config = Self {
            uri: toml.uri.clone(),
            database: toml.database.clone(),
            host_collection: toml.host_collection.clone(),
            timeout: Self::sanity_check_timeout(toml.timeout),
            native_collection: toml.native_collection.clone(),
        };

        config.check_mongo_connection().await
    }

    async fn check_mongo_connection(self) -> Result<Self, SentinelError> {
        self.get_db().await?.run_command(doc! {"ping": 1}, None).await?;
        debug!("Mongo connected successfully");
        Ok(self)
    }

    async fn get_db(&self) -> Result<Database, SentinelError> {
        let mut options = ClientOptions::parse(&self.uri).await?;
        options.server_selection_timeout = self.timeout;
        options.connect_timeout = self.timeout;
        let client = Client::with_options(options)?;
        Ok(client.database(&self.database))
    }

    pub async fn get_host_collection(&self) -> Result<Collection<HostOutput>, SentinelError> {
        debug!("Getting host mongo collection @ '{}'...", self.host_collection);
        let db = self.get_db().await?;
        Ok(db.collection(&self.host_collection))
    }

    pub async fn get_native_collection(&self) -> Result<Collection<NativeOutput>, SentinelError> {
        debug!("Getting native mongo collection @ '{}'...", self.native_collection);
        let db = self.get_db().await?;
        Ok(db.collection(&self.native_collection))
    }

    pub async fn get_heartbeats_collection(&self) -> Result<Collection<HeartbeatsJson>, SentinelError> {
        debug!("Getting heartbeats mongo collection...");
        let db = self.get_db().await?;
        Ok(db.collection("heartbeats"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn should_get_mongo_config() {
        match Config::new().await {
            Ok(_) => assert!(true),
            Err(SentinelError::MongoDb(e)) => panic!("error getting config: {e}"),
            Err(e) => panic!("wrong type of error received: {e}"),
        }
    }

    #[tokio::test]
    async fn should_get_host_collection() {
        let mongo_config = Config::new().await.unwrap().mongo_config;
        mongo_config.get_host_collection().await.unwrap();
    }

    #[tokio::test]
    async fn should_get_native_collection() {
        let mongo_config = Config::new().await.unwrap().mongo_config;
        mongo_config.get_native_collection().await.unwrap();
    }
}
