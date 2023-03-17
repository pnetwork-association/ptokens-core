use std::{
    result::Result,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::SentinelError;

pub fn get_utc_timestamp() -> Result<u128, SentinelError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())
}
