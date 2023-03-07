#[derive(Debug)]
pub enum SentinelError {
    TimeoutError(String),
    CommonError(common::AppError),
    ConfigError(crate::config::Error),
    SerdeJsonError(serde_json::Error),
    MongoDbError(mongodb::error::Error),
    BatchingError(crate::batching::Error),
    ParseIntError(std::num::ParseIntError),
    EndpointError(crate::endpoints::Error),
    LoggerError(flexi_logger::FlexiLoggerError),
    JsonRpcError(jsonrpsee::core::error::Error),
}

impl std::fmt::Display for SentinelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::CommonError(ref err) => write!(f, "{err}"),
            Self::JsonRpcError(ref err) => write!(f, "{err}"),
            Self::LoggerError(ref err) => write!(f, "logger error: {err}"),
            Self::TimeoutError(ref err) => write!(f, "timeout error: {err}"),
            Self::MongoDbError(ref err) => write!(f, "mongodb error: {err}"),
            Self::EndpointError(ref err) => write!(f, "endpoint error: {err}"),
            Self::BatchingError(ref err) => write!(f, "batching error: {err}"),
            Self::ParseIntError(ref err) => write!(f, "parse int error: {err}"),
            Self::ConfigError(ref err) => write!(f, "configuration error: {err}"),
            Self::SerdeJsonError(ref err) => write!(f, "serde json error: {err}"),
        }
    }
}

impl std::error::Error for SentinelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::ConfigError(_) => None,
            Self::TimeoutError(_) => None,
            Self::EndpointError(_) => None,
            Self::BatchingError(_) => None,
            Self::CommonError(ref err) => Some(err),
            Self::LoggerError(ref err) => Some(err),
            Self::JsonRpcError(ref err) => Some(err),
            Self::MongoDbError(ref err) => Some(err),
            Self::ParseIntError(ref err) => Some(err),
            Self::SerdeJsonError(ref err) => Some(err),
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

impl From<serde_json::Error> for SentinelError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJsonError(err)
    }
}

impl From<std::num::ParseIntError> for SentinelError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}
