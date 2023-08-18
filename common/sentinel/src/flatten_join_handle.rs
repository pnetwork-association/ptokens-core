use std::result::Result;

use tokio::task::JoinHandle;

use crate::SentinelError;

// NOTE: From here: https://docs.rs/tokio/latest/tokio/macro.select.html
pub async fn flatten_join_handle(handle: JoinHandle<Result<(), SentinelError>>) -> Result<(), SentinelError> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(_err) => Err(SentinelError::Custom("join handle flattening failed".into())),
    }
}
