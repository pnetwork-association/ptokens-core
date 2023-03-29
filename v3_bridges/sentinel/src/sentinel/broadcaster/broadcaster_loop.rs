use std::result::Result;

use lib::{BroadcasterMessages, MongoMessages, SentinelConfig, SentinelError};
use tokio::{
    sync::mpsc::{Receiver as MpscRx, Sender as MpscTx},
    time::{sleep, Duration},
};

const MONGO_CHECK_SLEEP_TIME: u64 = 2 * 1000; // TODO get from config

async fn main_loop(mut mongo_tx: MpscTx<MongoMessages>) -> Result<(), SentinelError> {
    let mut loop_counter = 0;

    'broadcaster_loop: loop {
        sleep(Duration::from_millis(MONGO_CHECK_SLEEP_TIME)).await;
        loop_counter += 1;
        continue 'broadcaster_loop;
    }
}

pub async fn broadcaster_loop(
    rx: MpscRx<BroadcasterMessages>,
    mongo_tx: MpscTx<MongoMessages>,
    config: SentinelConfig,
    disable_broadcaster: bool,
) -> Result<(), SentinelError> {
    let _host_endpoints = config.get_host_endpoints();
    let _native_endpoints = config.get_native_endpoints();

    if disable_broadcaster {
        warn!("Broadcaster has been disabled");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                warn!("broadcaster shutting down...");
                Err(SentinelError::SigInt("broadcaster".into()))
            },
        }
    } else {
        warn!("Broadcaster loop running...");
        return tokio::select! {
            r = main_loop(mongo_tx) => r,
            _ = tokio::signal::ctrl_c() => {
                warn!("broadcaster shutting down...");
                Err(SentinelError::SigInt("broadcaster".into()))
            },
        };
    }
}
