use thiserror::Error;

use super::NetworkId;

#[derive(Debug, Error)]
pub enum NetworkIdError {
    #[error("unsupported network id: {0}")]
    Unsupported(NetworkId),

    #[error("network id app error: {0}")]
    AppError(#[from] common::AppError),

    #[error("invalid network ID '{0}'")]
    InvalidNetworkId(String),

    #[error("invalid protocol ID '{0}'")]
    InvalidProtocolId(String),

    #[error("not enough bytes to createn network ID, expected {expected}, got {got}'")]
    NotEnoughBytes { expected: usize, got: usize },

    #[error("cannot convert from network id '{from}' to {to}")]
    CannotConvert { from: NetworkId, to: String },
}
