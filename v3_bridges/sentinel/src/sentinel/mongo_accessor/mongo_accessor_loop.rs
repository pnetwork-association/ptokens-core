use std::result::Result;

use lib::{HeartbeatsJson, HostOutput, MongoAccessorMessages, MongoConfig, NativeOutput, SentinelError};
use mongodb::{bson::doc, Collection};
use tokio::sync::mpsc::Receiver as MpscRx;

async fn insert_into_mongodb<T: std::fmt::Display + serde::Serialize>(
    output: T,
    collection: &Collection<T>,
) -> Result<(), SentinelError> {
    info!("Adding output to mongo: {output}");
    let insert_options = None;
    collection.insert_one(output, insert_options).await?;
    Ok(())
}

async fn update_heartbeat(h: &HeartbeatsJson, collection: &Collection<HeartbeatsJson>) -> Result<(), SentinelError> {
    let f = doc! {"_id":"heartbeats"};
    collection.delete_one(f, None).await?;
    collection.insert_one(h, None).await?;
    Ok(())
}

pub async fn mongo_accessor_loop(
    mongo_config: MongoConfig,
    mut mongo_accessor_rx: MpscRx<MongoAccessorMessages>,
) -> Result<(), SentinelError> {
    info!("mongo accessor listening...");
    let host_collection = mongo_config.get_host_collection().await?;
    let native_collection = mongo_config.get_native_collection().await?;
    let heartbeats_collection = mongo_config.get_heartbeats_collection().await?;
    update_heartbeat(&HeartbeatsJson::default(), &heartbeats_collection).await?;

    'mongo_accessor_loop: loop {
        tokio::select! {
            r = mongo_accessor_rx.recv() => match r {
                Some(MongoAccessorMessages::PutNative(msg)) => {
                    insert_into_mongodb(msg, &native_collection).await?;
                    continue 'mongo_accessor_loop
                },
                Some(MongoAccessorMessages::PutHost(msg)) => {
                    insert_into_mongodb(msg, &host_collection).await?;
                    continue 'mongo_accessor_loop
                },
                Some(MongoAccessorMessages::PutHeartbeats(msg)) => {
                    update_heartbeat(&msg, &heartbeats_collection).await?;
                    continue 'mongo_accessor_loop
                },
                None => {
                    let m = "all mongo accessor senders dropped!";
                    warn!("{m}");
                    break 'mongo_accessor_loop Err(SentinelError::Custom(m.into()))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("mongo accessor shutting down...");
                break 'mongo_accessor_loop Err(SentinelError::SigInt("mongo accessor".into()))
            },
        }
    }
}
