use std::result::Result;

use lib::{MongoAccessorMessages, SentinelError};
use tokio::sync::mpsc::Receiver as MpscRx;

async fn process_message(_msg: MongoAccessorMessages) -> Result<(), SentinelError> {
    Ok(())
}

pub async fn mongo_accessor_loop(mut mongo_accessor_rx: MpscRx<MongoAccessorMessages>) -> Result<(), SentinelError> {
    info!("mongo accessor listening...");

    'mongo_accessor_loop: loop {
        tokio::select! {
            r = mongo_accessor_rx.recv() => {
                if let Some(msg) = r {
                    process_message(msg).await?;
                    continue 'mongo_accessor_loop
                } else {
                    let m = "all mongo accessor senders dropped!";
                    warn!("{m}");
                    break 'mongo_accessor_loop Err(SentinelError::Custom(m.into()))
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("mongo accessor shutting down...");
                break 'mongo_accessor_loop Err(SentinelError::SigInt("core accessor".into()))
            },
        }
    }
}
