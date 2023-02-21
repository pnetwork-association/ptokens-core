use std::fs::write;

use anyhow::Result;

pub fn write_file(s: &str, path: &str) -> Result<()> {
    info!("[+] Writing file to path {path}");
    Ok(write(path, s.as_bytes())?)
}
