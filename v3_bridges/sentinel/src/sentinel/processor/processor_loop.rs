use std::result::Result;

use lib::{BroadcastMessages, ProcessorMessages, SentinelError, SyncerMessages};
use tokio::sync::broadcast::{Receiver, Sender};

use crate::sentinel::processor::{process_host_batch, process_native_batch};

/*
NOTE: can drain a channel in a loop like so:
while let Ok(value) = receiver.try_recv() {
    println!("received {}", value);
}
 */

pub async fn processor_loop(
    mut broadcast_rx: Receiver<BroadcastMessages>,
    mut processor_rx: Receiver<ProcessorMessages>,
    _host_syncer_tx: Sender<SyncerMessages>,
    _native_syncer_tx: Sender<SyncerMessages>,
) -> Result<(), SentinelError> {
    info!("Starting processor loop...");

    'processor_loop: loop {
        tokio::select! {
            r = processor_rx.recv() => {
                match r {
                    Ok(ProcessorMessages::ProcessNative(material)) => {
                        debug!("Processing native material...");
                        // TODO if there are errors in processing, we need to handle them by
                        // draining the channel then telling the syncer to restart.
                        // Could be racy?
                        // Could pause the syncer first
                        // Then drain the channel
                        // Then give it a restart command with the correct block?
                        // Probably need a dedicated channel between the syncer and the processor,
                        // including for each side (native & host), hmmmm. enum SyncerProcessorMessages ??
                        process_native_batch(&material)?;
                        continue 'processor_loop;
                    },
                    Ok(ProcessorMessages::ProcessHost(material)) => {
                        debug!("Processing host material...");
                        process_host_batch(&material)?;
                        continue 'processor_loop;
                    },
                    Err(e) => {
                        warn!("processor receiver error: {e}!");
                        return Err(e.into())
                    }
                }
            },
            r = broadcast_rx.recv() => {
                match r {
                    Ok(BroadcastMessages::Shutdown) => {
                        warn!("processor gracefully shutting down...");
                        return Ok::<(), SentinelError>(());
                    },
                    Err(e) => {
                        warn!("processor receiver error: {e}!");
                        return Err(e.into());
                    },
                }
            },
        }
    }
}
