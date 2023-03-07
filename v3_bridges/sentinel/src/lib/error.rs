#[derive(Debug)]
pub enum SentinelError {
    /// Represents a failture to get a given block number.
    NoBlock(u64),

    /// Represents an error originating from the common crates.
    CommonError(common::AppError),

    /// Represents an error originating from the json RPC crate.
    JsonRpcError(jsonrpsee::core::error::Error),

    /// Represents an error originating from configuration file.
    ConfigError(crate::config::Error),

    /// Represents an error due to something timing out.
    TimeoutError(String),

    /// Represents a batching error.
    BatchingError(crate::batching::Error),

    /// Represents a mongodb error.
    MongoDbError(mongodb::error::Error),

    /// A logging crate error
    LoggerError(flexi_logger::FlexiLoggerError),
}

impl std::fmt::Display for SentinelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::CommonError(ref err) => write!(f, "{err}"),
            Self::JsonRpcError(ref err) => write!(f, "{err}"),
            Self::LoggerError(ref err) => write!(f, "Logger error: {err}"),
            Self::TimeoutError(ref err) => write!(f, "Timeout error: {err}"),
            Self::MongoDbError(ref err) => write!(f, "Mongodb error: {err}"),
            Self::BatchingError(ref err) => write!(f, "Batching error: {err}"),
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
            Self::TimeoutError(_) => None,
            Self::BatchingError(_) => None,
            Self::CommonError(ref err) => Some(err),
            Self::LoggerError(ref err) => Some(err),
            Self::JsonRpcError(ref err) => Some(err),
            Self::MongoDbError(ref err) => Some(err),
        }
    }
}

impl From<common::errors::AppError> for SentinelError {
    fn from(err: common::errors::AppError) -> Self {
        Self::CommonError(err)
    }
}

impl From<tokio::time::error::Elapsed> for SentinelError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Self::TimeoutError(err.to_string())
    }
}

impl From<jsonrpsee::core::Error> for SentinelError {
    fn from(err: jsonrpsee::core::Error) -> Self {
        Self::JsonRpcError(err)
    }
}

impl From<mongodb::error::Error> for SentinelError {
    fn from(err: mongodb::error::Error) -> Self {
        Self::MongoDbError(err)
    }
}

impl From<flexi_logger::FlexiLoggerError> for SentinelError {
    fn from(err: flexi_logger::FlexiLoggerError) -> Self {
        Self::LoggerError(err)
    }
}
