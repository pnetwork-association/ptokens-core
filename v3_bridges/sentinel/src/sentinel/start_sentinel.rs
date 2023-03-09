use std::result::Result;

use futures::join;
use lib::{Batch, BroadcastMessages, SentinelConfig, SentinelError};
use serde_json::json;
use tokio::{
    signal,
    sync::{
        broadcast,
        broadcast::{Receiver, Sender},
    },
};

use crate::sentinel::{processor_loop, syncer_loop};

pub async fn start_sentinel(config: &SentinelConfig) -> Result<String, SentinelError> {
    // NOTE: Set up our channels...
    let (tx_1, rx_1): (Sender<BroadcastMessages>, Receiver<BroadcastMessages>) = broadcast::channel(1337);
    let tx_2 = tx_1.clone();
    let tx_3 = tx_1.clone();
    let rx_2 = tx_1.subscribe();
    let rx_3 = tx_1.subscribe();

    // NOTE: Set up our batches...
    let batch_1 = Batch::new_from_config(true, &config)?;
    let batch_2 = Batch::new_from_config(false, &config)?;

    // NOTE: Hand everything off to async threads...
    let thread_1 = tokio::spawn(async move { syncer_loop(batch_1, tx_1, rx_1).await });
    let thread_2 = tokio::spawn(async move { syncer_loop(batch_2, tx_2, rx_2).await });
    let thread_3 = tokio::spawn(async move { processor_loop(rx_3).await });

    // NOTE: Graceful shutdown upon ctrl-c...
    match signal::ctrl_c().await {
        Ok(()) => {
            warn!("Sigint caught, shutting down gracefully...");
            tx_3.send(BroadcastMessages::Shutdown)
        },
        Err(err) => {
            warn!("Unable to listen for shutdown signal: {err} - shutting down as a precaution!");
            tx_3.send(BroadcastMessages::Shutdown)
        },
    }?;

    let (res_1, res_2, _res_3) = join!(thread_1, thread_2, thread_3);
    let thread_1_result = res_1??;
    let thread_2_result = res_2??;
    let res = json!({
        "jsonrpc": "2.0",
        "result": {
            "thread_1": thread_1_result,
            "thread_2": thread_2_result,
        },
    })
    .to_string();
    Ok(res)
}
