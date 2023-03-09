use std::result::Result;

use futures::join;
use lib::{Batch, SentinelConfig, SentinelError};
use serde_json::json;
use tokio::{
    signal,
    sync::{
        broadcast,
        broadcast::{Receiver, Sender},
    },
};

use crate::sentinel::{processor_loop, syncer_loop};

// TODO need a broadcast channel sending to all threads to allow graceful shutdown etc
// TODO need mspc chennel for processor so syncers can send batches to it for processsing.

pub async fn start_sentinel(config: &SentinelConfig) -> Result<String, SentinelError> {
    let (tx, rx_1): (Sender<bool>, Receiver<bool>) = broadcast::channel(1337);
    let rx_2 = tx.subscribe();
    let rx_3 = tx.subscribe();

    let batch_1 = Batch::new_from_config(true, &config)?;
    let batch_2 = Batch::new_from_config(false, &config)?;

    let thread_1 = tokio::spawn(async move { syncer_loop(batch_1, rx_1).await });
    let thread_2 = tokio::spawn(async move { syncer_loop(batch_2, rx_2).await });
    let thread_3 = tokio::spawn(async move { processor_loop(rx_3).await });

    // NOTE: Graceful shutdown upon ctrl-c...
    match signal::ctrl_c().await {
        Ok(()) => {
            warn!("ctrl-c caught, shutting down gracefully, please wait...");
            tx.send(true)
            // TODO send shutdown signal to application and wait
        },
        Err(err) => {
            warn!(
                "Unable to listen for shutdown signal: {} - shutting down as a precaution!",
                err
            );
            tx.send(true)
            // we also shut down in case of error
        },
    }?;

    let (res_1, res_2, res_3) = join!(thread_1, thread_2, thread_3);
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
