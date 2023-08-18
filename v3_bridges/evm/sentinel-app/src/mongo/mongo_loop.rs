use std::result::Result;

use common_sentinel::{HeartbeatsJson, MongoConfig, MongoMessages, SentinelError};
use mongodb::{bson::doc, options::FindOneAndReplaceOptions, Collection};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    sync::mpsc::Receiver as MpscRx,
    time::{sleep, Duration},
};

const MONGO_RETRY_SLEEP_TIME: u64 = 500;

#[allow(unused)]
async fn insert_into_mongodb<T: std::fmt::Display + serde::Serialize>(
    output: T,
    collection: &Collection<T>,
) -> Result<(), SentinelError> {
    debug!("adding output to mongo: {output}");
    loop {
        match collection.insert_one(&output, None).await {
            Ok(_) => break Ok(()),
            Err(ref e) if e.contains_label(mongodb::error::RETRYABLE_WRITE_ERROR) => {
                warn!("error writing to mongo, sleeing {MONGO_RETRY_SLEEP_TIME}ms and retrying...");
                sleep(Duration::from_millis(MONGO_RETRY_SLEEP_TIME)).await;
                continue;
            },
            Err(e) => break Err(e.into()),
        }
    }
}

async fn update_heartbeat(h: &HeartbeatsJson, collection: &Collection<HeartbeatsJson>) -> Result<(), SentinelError> {
    loop {
        let f = doc! {"_id":"heartbeats"};
        let o = FindOneAndReplaceOptions::builder().upsert(true).build();
        match collection.find_one_and_replace(f, h, o).await {
            Ok(_) => break Ok(()),
            Err(ref e) if e.contains_label(mongodb::error::RETRYABLE_WRITE_ERROR) => {
                warn!("error writing heartbeat to mongo, sleeing {MONGO_RETRY_SLEEP_TIME}ms and retrying...");
                sleep(Duration::from_millis(MONGO_RETRY_SLEEP_TIME)).await;
                continue;
            },
            Err(e) => break Err(e.into()),
        }
    }
}

#[allow(unused)]
async fn update_in_mongodb<T>(t: &T, id: &str, collection: &Collection<T>) -> Result<(), SentinelError>
where
    T: std::fmt::Display + Serialize + DeserializeOwned,
{
    loop {
        let f = doc! { "_id": id };
        let o = FindOneAndReplaceOptions::builder().upsert(true).build();

        match collection.find_one_and_replace(f, t, o).await {
            Ok(_) => break Ok(()),
            Err(ref e) if e.contains_label(mongodb::error::RETRYABLE_WRITE_ERROR) => {
                warn!("error writing `{id}` to mongo, sleeing {MONGO_RETRY_SLEEP_TIME}ms and retrying...");
                sleep(Duration::from_millis(MONGO_RETRY_SLEEP_TIME)).await;
                continue;
            },
            Err(e) => break Err(e.into()),
        }
    }
}

async fn get_heartbeats(collection: &Collection<HeartbeatsJson>) -> Result<HeartbeatsJson, SentinelError> {
    let f = doc! {"_id":"heartbeats"};
    Ok(collection.find_one(f, None).await?.unwrap_or_default())
}

#[allow(unused)]
async fn get<T>(key: &str, collection: &Collection<T>) -> Result<T, SentinelError>
where
    T: std::fmt::Display + Serialize + DeserializeOwned + Send + Unpin + Sync + Default,
{
    let f = doc! {"_id": key};
    Ok(collection.find_one(f, None).await?.unwrap_or_default())
}

async fn handle_message(msg: MongoMessages, collection: &Collection<HeartbeatsJson>) -> Result<(), SentinelError> {
    match msg {
        MongoMessages::PutHeartbeats(msg) => {
            update_heartbeat(&msg, collection).await?;
        },
        MongoMessages::GetHeartbeats(responder) => {
            let r = get_heartbeats(collection).await;
            let _ = responder.send(r);
        },
    }
    Ok(())
}

pub async fn mongo_loop(mongo_config: MongoConfig, mut mongo_rx: MpscRx<MongoMessages>) -> Result<(), SentinelError> {
    info!("checking mongo config...");
    mongo_config.check_mongo_connection().await?;
    info!("mongo listening");

    let heartbeats_collection = mongo_config.get_heartbeats_collection().await?;
    update_heartbeat(&HeartbeatsJson::default(), &heartbeats_collection).await?;

    'mongo_loop: loop {
        tokio::select! {
            r = mongo_rx.recv() => match r {
                Some(msg) => {
                    handle_message(msg, &heartbeats_collection).await?;
                    continue 'mongo_loop
                },
                None => {
                    let m = "all mongo senders dropped";
                    warn!("{m}");
                    break 'mongo_loop Err(SentinelError::Custom(m.into()))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("mongo shutting down...");
                break 'mongo_loop Err(SentinelError::SigInt("mongo".into()))
            },
        }
    }
}
