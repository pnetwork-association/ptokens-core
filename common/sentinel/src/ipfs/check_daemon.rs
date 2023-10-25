use std::{
    process::Command,
    str::{from_utf8, FromStr},
};

use serde::{Deserialize, Serialize};

use super::IpfsError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IpfsIdOutput {
    addresses: Option<Vec<String>>,
    protocols: Option<Vec<String>>,
}

impl FromStr for IpfsIdOutput {
    type Err = IpfsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

pub fn check_ipfs_daemon_is_running(ipfs_bin_path: &str) -> Result<(), IpfsError> {
    info!("checking ipfs daemon is running...");
    let output = Command::new(ipfs_bin_path).arg("id").output()?;

    if !output.status.success() {
        return Err(IpfsError::CmdFailed(from_utf8(&output.stderr)?.into()));
    }

    let parsed_output = IpfsIdOutput::from_str(from_utf8(&output.stdout)?)?;

    if parsed_output.addresses.is_none() && parsed_output.protocols.is_none() {
        error!("ipfs daemon not running: {parsed_output:?}");
        Err(IpfsError::DaemonNotRunning)
    } else {
        info!("ipfs daemon appears to be running correctly");
        Ok(())
    }
}
