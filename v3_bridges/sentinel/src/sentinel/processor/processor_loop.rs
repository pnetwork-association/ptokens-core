use std::result::Result;

use lib::{handle_sigint, SentinelError};
use tokio::{
    sync::broadcast::Receiver,
    time::{sleep, Duration},
};

async fn main_loop(_log_prefix: &str) -> Result<(), SentinelError> {
    let mut i = 0;

    'main: loop {
        info!("Processor loop #{i}");
        sleep(Duration::from_millis(10_000)).await;
        i += 1;
        continue 'main;
    }
}

pub async fn processor_loop(rx: Receiver<bool>) -> Result<(), SentinelError> {
    info!("Starting processor loop...");
    let log_prefix = "processor";

    tokio::select! {
        _ = main_loop(log_prefix) => Ok(()),
        _ = handle_sigint(log_prefix, rx) => Ok(())
    }
}
