use crate::{BroadcastMessages, ProcessorMessages, SyncerMessages};

#[derive(Debug)]
pub enum SentinelError {
    Custom(String),
    Timeout(String),
    Common(common::AppError),
    Config(config::ConfigError),
    SerdeJson(serde_json::Error),
    MongoDb(mongodb::error::Error),
    Batching(crate::batching::Error),
    ParseInt(std::num::ParseIntError),
    Endpoint(crate::endpoints::Error),
    TokioJoin(tokio::task::JoinError),
    SentinelConfig(crate::config::Error),
    Logger(flexi_logger::FlexiLoggerError),
    JsonRpc(jsonrpsee::core::error::Error),
    RocksDb(common_rocksdb::RocksdbDatabaseError),
    Receiver(tokio::sync::broadcast::error::RecvError),
    SyncerChannel(Box<tokio::sync::broadcast::error::SendError<SyncerMessages>>),
    ProcessorChannel(Box<tokio::sync::broadcast::error::SendError<ProcessorMessages>>),
    BroadcastChannel(Box<tokio::sync::broadcast::error::SendError<BroadcastMessages>>),
}

impl std::fmt::Display for SentinelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Custom(ref e) => write!(f, "{e}"),
            Self::Common(ref err) => write!(f, "{err}"),
            Self::JsonRpc(ref err) => write!(f, "{err}"),
            Self::RocksDb(ref err) => write!(f, "{err}"),
            Self::Config(ref err) => write!(f, "config error: {err}"),
            Self::Logger(ref err) => write!(f, "logger error: {err}"),
            Self::Timeout(ref err) => write!(f, "timeout error: {err}"),
            Self::MongoDb(ref err) => write!(f, "mongodb error: {err}"),
            Self::Endpoint(ref err) => write!(f, "endpoint error: {err}"),
            Self::Batching(ref err) => write!(f, "batching error: {err}"),
            Self::ParseInt(ref err) => write!(f, "parse int error: {err}"),
            Self::SerdeJson(ref err) => write!(f, "serde json error: {err}"),
            Self::TokioJoin(ref err) => write!(f, "tokio join error: {err}"),
            Self::Receiver(ref err) => write!(f, "tokio receive error: {err}"),
            Self::SyncerChannel(ref err) => write!(f, "syncer channel error: {err}"),
            Self::BroadcastChannel(ref err) => write!(f, "broadcast channel error: {err}"),
            Self::ProcessorChannel(ref err) => write!(f, "processor channel error: {err}"),
            Self::SentinelConfig(ref err) => write!(f, "sentinel configuration error: {err}"),
        }
    }
}

impl std::error::Error for SentinelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Custom(_) => None,
            Self::Timeout(_) => None,
            Self::Endpoint(_) => None,
            Self::Batching(_) => None,
            Self::Common(ref err) => Some(err),
            Self::Config(ref err) => Some(err),
            Self::Logger(ref err) => Some(err),
            Self::JsonRpc(ref err) => Some(err),
            Self::MongoDb(ref err) => Some(err),
            Self::RocksDb(ref err) => Some(err),
            Self::Receiver(ref err) => Some(err),
            Self::ParseInt(ref err) => Some(err),
            Self::TokioJoin(ref err) => Some(err),
            Self::SerdeJson(ref err) => Some(err),
            Self::SyncerChannel(ref err) => Some(err),
            Self::SentinelConfig(ref err) => Some(err),
            Self::BroadcastChannel(ref err) => Some(err),
            Self::ProcessorChannel(ref err) => Some(err),
        }
    }
}

impl From<common::errors::AppError> for SentinelError {
    fn from(err: common::errors::AppError) -> Self {
        Self::Common(err)
    }
}

impl From<tokio::time::error::Elapsed> for SentinelError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(err.to_string())
    }
}

impl From<jsonrpsee::core::Error> for SentinelError {
    fn from(err: jsonrpsee::core::Error) -> Self {
        Self::JsonRpc(err)
    }
}

impl From<mongodb::error::Error> for SentinelError {
    fn from(err: mongodb::error::Error) -> Self {
        Self::MongoDb(err)
    }
}

impl From<flexi_logger::FlexiLoggerError> for SentinelError {
    fn from(err: flexi_logger::FlexiLoggerError) -> Self {
        Self::Logger(err)
    }
}

impl From<serde_json::Error> for SentinelError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}

impl From<std::num::ParseIntError> for SentinelError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::ParseInt(err)
    }
}

impl From<tokio::task::JoinError> for SentinelError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::TokioJoin(err)
    }
}

impl From<crate::config::Error> for SentinelError {
    fn from(err: crate::config::Error) -> Self {
        Self::SentinelConfig(err)
    }
}

impl From<config::ConfigError> for SentinelError {
    fn from(err: config::ConfigError) -> Self {
        Self::Config(err)
    }
}

impl From<tokio::sync::broadcast::error::SendError<BroadcastMessages>> for SentinelError {
    fn from(err: tokio::sync::broadcast::error::SendError<BroadcastMessages>) -> Self {
        Self::BroadcastChannel(Box::new(err))
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for SentinelError {
    fn from(err: tokio::sync::broadcast::error::RecvError) -> Self {
        Self::Receiver(err)
    }
}

impl From<tokio::sync::broadcast::error::SendError<SyncerMessages>> for SentinelError {
    fn from(err: tokio::sync::broadcast::error::SendError<SyncerMessages>) -> Self {
        Self::SyncerChannel(Box::new(err))
    }
}

impl From<tokio::sync::broadcast::error::SendError<ProcessorMessages>> for SentinelError {
    fn from(err: tokio::sync::broadcast::error::SendError<ProcessorMessages>) -> Self {
        Self::ProcessorChannel(Box::new(err))
    }
}

impl From<common_rocksdb::RocksdbDatabaseError> for SentinelError {
    fn from(err: common_rocksdb::RocksdbDatabaseError) -> Self {
        Self::RocksDb(err)
    }
}