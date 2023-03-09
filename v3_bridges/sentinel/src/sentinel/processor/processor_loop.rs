use std::result::Result;

use lib::{handle_sigint, BroadcastMessages, SentinelError};
use tokio::{
    sync::broadcast::Receiver,
    time::{sleep, Duration},
};

pub async fn processor_loop(mut rx: Receiver<BroadcastMessages>) -> Result<(), SentinelError> {
    info!("Starting processor loop...");

    'processor_loop: loop {
        match rx.recv().await {
            Ok(BroadcastMessages::ProcessNative(batch)) => {
                debug!("processing native batch...");
                // process it...
                continue 'processor_loop;
            },
            Ok(BroadcastMessages::ProcessHost(batch)) => {
                debug!("processing host batch...");
                // process it...
                continue 'processor_loop;
            },
            Ok(BroadcastMessages::Shutdown) => {
                warn!("Processor gracefully shutting down!");
                return Ok::<(), SentinelError>(());
            },
            Err(e) => {
                warn!("Processor reciver error: {e}!");
                return Err(e.into());
            },
        }
    }
}
