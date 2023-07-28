use std::{result::Result, time::Duration};

use mongodb::{bson::doc, options::ClientOptions, Client, Collection, Database};
use serde::Deserialize;

use crate::{HeartbeatsJson, SentinelError, MILLISECONDS_MULTIPLIER};

#[derive(Debug, Clone, Deserialize)]
pub struct MongoToml {
    uri: String,
    timeout: u32,
    database: String,
    sleep_duration: u64,
}

#[derive(Debug, Clone)]
pub struct MongoConfig {
    uri: String,
    database: String,
    sleep_duration: u64,
    timeout: Option<Duration>,
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

    pub fn from_toml(toml: &MongoToml) -> Self {
        Self {
            uri: toml.uri.clone(),
            database: toml.database.clone(),
            timeout: Self::sanity_check_timeout(toml.timeout),
            sleep_duration: toml.sleep_duration * MILLISECONDS_MULTIPLIER,
        }
    }

    pub async fn check_mongo_connection(&self) -> Result<(), SentinelError> {
        let db = self.get_db().await?;
        match db.run_command(doc! {"ping": 1}, None).await {
            Ok(_) => {
                debug!("Mongo connected successfully");
                Ok(())
            },
            Err(e) => {
                warn!("Could not connect to mongo db - please check your config!");
                Err(e.into())
            },
        }
    }

    async fn get_db(&self) -> Result<Database, SentinelError> {
        let mut options = ClientOptions::parse(&self.uri).await?;
        options.server_selection_timeout = self.timeout;
        options.connect_timeout = self.timeout;
        let client = Client::with_options(options)?;
        Ok(client.database(&self.database))
    }

    pub async fn get_heartbeats_collection(&self) -> Result<Collection<HeartbeatsJson>, SentinelError> {
        debug!("Getting heartbeats mongo collection...");
        let db = self.get_db().await?;
        Ok(db.collection("heartbeats"))
    }

    pub fn sleep_duration(&self) -> u64 {
        self.sleep_duration
    }
}
