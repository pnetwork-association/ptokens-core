use std::result::Result;

use tokio::sync::broadcast::Receiver;

use crate::SentinelError;

// NOTE: This can be used to "race" against infinite async loops, using `tokio::select!`, arresting
// those loops immediately.
//
// NOTE: This should not be used if you have any work to do to affect a graceful shutdown.
//
// The `main` function watches for sigints and if caught sends a signal down the broadcast channel
// that this function takes as an arg..
pub async fn handle_sigint(log_prefix: &str, mut rx: Receiver<bool>) -> Result<(), SentinelError> {
    rx.recv()
        .await
        .expect("Message in shutdown signal is irrelevant, thus ignored here!");
    // NOTE: The above yields until something comes down the pipe.
    warn!("{log_prefix} shutting down!");
    Ok(())
}
