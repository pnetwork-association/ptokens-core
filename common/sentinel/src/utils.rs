use std::{
    result::Result,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::SentinelError;

pub fn get_utc_timestamp() -> Result<u64, SentinelError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub async fn run_timer(t: u64) -> Result<(), SentinelError> {
    tokio::time::sleep(tokio::time::Duration::from_millis(t)).await;
    Ok(())
}
