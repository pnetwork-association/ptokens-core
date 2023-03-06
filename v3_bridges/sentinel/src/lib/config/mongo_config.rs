use std::str::FromStr;

use anyhow::Result;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client,
    Collection,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MongoConfig {
    uri: String,
    database: String,
    collection: String,
}

impl MongoConfig {
    pub async fn get_collection(&self) -> Result<Collection<Document>> {
        info!("Getting mongo collection '{}'...", self.collection);
        let client = Client::with_options(ClientOptions::parse(&self.uri).await?)?;
        let db = client.database(&self.database);
        Ok(db.collection::<Document>(&self.collection))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn should_get_collection() {
        let mongo_config = Config::new().unwrap().mongo_config;
        mongo_config.get_collection().await.unwrap();
    }
}
