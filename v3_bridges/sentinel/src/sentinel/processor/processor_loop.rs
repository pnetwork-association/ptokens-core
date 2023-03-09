use std::result::Result;

use lib::{BroadcastMessages, SentinelError};
use tokio::sync::broadcast::Receiver;

use super::{process_host_batch, process_native_batch};

pub async fn processor_loop(mut rx: Receiver<BroadcastMessages>) -> Result<(), SentinelError> {
    info!("Starting processor loop...");

    'processor_loop: loop {
        match rx.recv().await {
            Ok(BroadcastMessages::ProcessNative(material)) => {
                debug!("Processing native material...");
                process_native_batch(&material)?;
                continue 'processor_loop;
            },
            Ok(BroadcastMessages::ProcessHost(material)) => {
                debug!("Processing host material...");
                process_host_batch(&material)?;
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
