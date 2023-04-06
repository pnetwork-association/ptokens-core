use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkIdError {
    #[error("invalid network ID '{0}'")]
    InvalidNetworkId(String),

    #[error("invalid protocol ID '{0}'")]
    InvalidProtocolId(String),

    #[error("not enough bytes to createn network ID, expected {expected}, got {got}'")]
    NotEnoughBytes { expected: usize, got: usize },
}
