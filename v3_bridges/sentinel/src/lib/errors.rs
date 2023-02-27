#[derive(Debug)]
pub enum SentinelError {
    /// Represents a failture to get a given block number.
    NoBlock(u64),

    /// Represents an error originating from the common crates.
    CommonError(common::AppError),

    /// Represents an error originating from the json RPC crate.
    JsonRpcError(jsonrpsee::core::error::Error),

    /// Represents an error originating from configuration file.
    ConfigError(String),
}

impl std::fmt::Display for SentinelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::CommonError(ref err) => write!(f, "{err}"),
            Self::JsonRpcError(ref err) => write!(f, "{err}"),
            Self::NoBlock(num) => write!(f, "Cannot get block {num} from rpc"),
            Self::ConfigError(ref err) => write!(f, "Configuration error: {err}"),
        }
    }
}

impl std::error::Error for SentinelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::NoBlock(_) => None,
            Self::ConfigError(_) => None,
            Self::CommonError(ref err) => Some(err),
            Self::JsonRpcError(ref err) => Some(err),
        }
    }
}

impl From<common::errors::AppError> for SentinelError {
    fn from(err: common::errors::AppError) -> Self {
        Self::CommonError(err)
    }
}
