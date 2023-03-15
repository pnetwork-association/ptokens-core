use std::result::Result;

use tokio::sync::broadcast::Receiver;

use crate::{BroadcasterMessages, SentinelError};

// NOTE: This can be used to "race" against infinite async loops, using `tokio::select!`, arresting
// those loops immediately.
//
// NOTE: This should not be used if you have any work to do to affect a graceful shutdown.
//
// The `main` function watches for sigints and if caught sends a signal down the broadcast channel
// that this function takes as an arg..
pub async fn handle_sigint(log_prefix: &str, mut rx: Receiver<BroadcasterMessages>) -> Result<(), SentinelError> {
    let mut i = 1;
    loop {
        trace!("shutdown handler loop: #{i}");

        match rx.recv().await {
            // NOTE: The await yields until something comes down the pipe.
            Ok(BroadcasterMessages::Shutdown) => {
                warn!("{log_prefix} shutting down!");
                return Ok::<(), SentinelError>(());
            },
            _ => {
                i += 1;
                continue;
            },
        }
    }
}
