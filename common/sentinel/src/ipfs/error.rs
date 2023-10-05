use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpfsError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error("shell command failed: {0}")]
    CmdFailed(String),

    #[error("ipfs daemon not appear to be running - please start one")]
    DaemonNotRunning,

    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("utf8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}
