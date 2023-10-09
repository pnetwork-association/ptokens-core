use std::{fs::File, io::prelude::*, process::Command, str::from_utf8};

use super::IpfsError;
use crate::SentinelStatus;

const IPFS_TOPIC: &str = "pnetwork-v3";
const IPFS_STATUS_PATH: &str = "./.temp.json";

fn write_temp_file(status: &SentinelStatus) -> Result<(), IpfsError> {
    let mut file = File::create(IPFS_STATUS_PATH)?;
    file.write_all(&serde_json::to_vec(status)?)?;

    Ok(())
}

pub fn publish_status(ipfs_bin_path: &str, status: SentinelStatus) -> Result<(), IpfsError> {
    debug!("publishing status...");

    write_temp_file(&status)?;

    let output = Command::new(ipfs_bin_path)
        .arg("pubsub")
        .arg("pub")
        .arg(IPFS_TOPIC)
        .arg(IPFS_STATUS_PATH)
        .output()?;

    if !output.status.success() {
        Err(IpfsError::CmdFailed(from_utf8(&output.stderr)?.into()))
    } else {
        debug!("status published successfully");
        Ok(())
    }
}
