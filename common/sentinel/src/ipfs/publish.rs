use std::{process::Command, str::from_utf8};

use super::IpfsError;
use crate::SentinelStatus;

const IPFS_TOPIC: &str = "pnetwork-v3";

pub fn publish_status(ipfs_bin_path: &str, status: SentinelStatus) -> Result<(), IpfsError> {
    debug!("publishing status...");

    let output = Command::new(ipfs_bin_path)
        .arg("pubsub")
        .arg("pub")
        .arg(IPFS_TOPIC)
        .arg(status.to_string())
        .output()?;

    if !output.status.success() {
        Err(IpfsError::CmdFailed(from_utf8(&output.stderr)?.into()))
    } else {
        debug!("status published successfully");
        Ok(())
    }
}
