use std::{fs::write, result::Result};

use lib::SentinelError;

pub fn write_file(s: &str, path: &str) -> Result<(), SentinelError> {
    info!("Writing file to path {path}");
    Ok(write(path, s.as_bytes())?)
}
