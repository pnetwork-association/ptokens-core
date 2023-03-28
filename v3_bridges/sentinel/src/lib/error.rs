use crate::{BroadcasterMessages, CoreMessages, MongoMessages, ProcessorMessages, SyncerMessages};
// FIXME Macro or something for the various channel errors?

#[derive(Debug)]
pub enum SentinelError {
    PoisonedLock,
    NoBlock(u64),
    Custom(String),
    SigInt(String),
    Timeout(String),
    SyncerRestart(u64),
    IO(std::io::Error),
    EthAbi(ethabi::Error),
    Json(serde_json::Value),
    Common(common::AppError),
    Config(config::ConfigError),
    SerdeJson(serde_json::Error),
    MongoDb(mongodb::error::Error),
    NoParent(common::NoParentError),
    Time(std::time::SystemTimeError),
    Batching(crate::batching::Error),
    ParseInt(std::num::ParseIntError),
    Endpoint(crate::endpoints::Error),
    TokioJoin(tokio::task::JoinError),
    SentinelConfig(crate::config::Error),
    Logger(flexi_logger::FlexiLoggerError),
    JsonRpc(jsonrpsee::core::error::Error),
    RocksDb(common_rocksdb::RocksdbDatabaseError),
    BlockAlreadyInDb(common::BlockAlreadyInDbError),
    Receiver(tokio::sync::broadcast::error::RecvError),
    OneshotReceiver(tokio::sync::oneshot::error::RecvError),
    CoreChannel(Box<tokio::sync::mpsc::error::SendError<CoreMessages>>),
    SyncerChannel(Box<tokio::sync::broadcast::error::SendError<SyncerMessages>>),
    MongoChannel(Box<tokio::sync::mpsc::error::SendError<MongoMessages>>),
    ProcessorChannel(Box<tokio::sync::mpsc::error::SendError<ProcessorMessages>>),
    BroadcastChannel(Box<tokio::sync::broadcast::error::SendError<BroadcasterMessages>>),
}

impl std::fmt::Display for SentinelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Json(ref e) => write!(f, "{e}"),
            Self::Custom(ref e) => write!(f, "{e}"),
            Self::Common(ref err) => write!(f, "{err}"),
            Self::EthAbi(ref err) => write!(f, "{err}"),
            Self::JsonRpc(ref err) => write!(f, "{err}"),
            Self::RocksDb(ref err) => write!(f, "{err}"),
            Self::NoParent(ref err) => write!(f, "{err}"),
            Self::IO(ref err) => write!(f, "IO error: {err}"),
            Self::BlockAlreadyInDb(ref err) => write!(f, "{err}"),
            Self::NoBlock(ref num) => write!(f, "no block {num}"),
            Self::PoisonedLock => write!(f, "posioned lock error!"),
            Self::Config(ref err) => write!(f, "config error: {err}"),
            Self::Logger(ref err) => write!(f, "logger error: {err}"),
            Self::Timeout(ref err) => write!(f, "timeout error: {err}"),
            Self::MongoDb(ref err) => write!(f, "mongodb error: {err}"),
            Self::Time(ref err) => write!(f, "system time error: {err}"),
            Self::Endpoint(ref err) => write!(f, "endpoint error: {err}"),
            Self::Batching(ref err) => write!(f, "batching error: {err}"),
            Self::ParseInt(ref err) => write!(f, "parse int error: {err}"),
            Self::SerdeJson(ref err) => write!(f, "serde json error: {err}"),
            Self::TokioJoin(ref err) => write!(f, "tokio join error: {err}"),
            Self::Receiver(ref err) => write!(f, "tokio receive error: {err}"),
            Self::CoreChannel(ref err) => write!(f, "core channel error: {err}"),
            Self::MongoChannel(ref err) => write!(f, "mongo channel error: {err}"),
            Self::SigInt(ref component) => write!(f, "sigint caught in {component}"),
            Self::SyncerChannel(ref err) => write!(f, "syncer channel error: {err}"),
            Self::OneshotReceiver(ref err) => write!(f, "oneshot receiver error: {err}"),
            Self::BroadcastChannel(ref err) => write!(f, "broadcast channel error: {err}"),
            Self::ProcessorChannel(ref err) => write!(f, "processor channel error: {err}"),
            Self::SyncerRestart(ref err) => write!(f, "syncer to restart from block {err}"),
            Self::SentinelConfig(ref err) => write!(f, "sentinel configuration error: {err}"),
        }
    }
}

impl std::error::Error for SentinelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Json(_) => None,
            Self::SigInt(_) => None,
            Self::Custom(_) => None,
            Self::NoBlock(_) => None,
            Self::Timeout(_) => None,
            Self::Endpoint(_) => None,
            Self::NoParent(_) => None,
            Self::Batching(_) => None,
            Self::PoisonedLock => None,
            Self::IO(ref err) => Some(err),
            Self::SyncerRestart(_) => None,
            Self::Time(ref err) => Some(err),
            Self::BlockAlreadyInDb(_) => None,
            Self::Common(ref err) => Some(err),
            Self::Config(ref err) => Some(err),
            Self::EthAbi(ref err) => Some(err),
            Self::Logger(ref err) => Some(err),
            Self::JsonRpc(ref err) => Some(err),
            Self::MongoDb(ref err) => Some(err),
            Self::RocksDb(ref err) => Some(err),
            Self::Receiver(ref err) => Some(err),
            Self::ParseInt(ref err) => Some(err),
            Self::TokioJoin(ref err) => Some(err),
            Self::SerdeJson(ref err) => Some(err),
            Self::MongoChannel(ref err) => Some(err),
            Self::SyncerChannel(ref err) => Some(err),
            Self::SentinelConfig(ref err) => Some(err),
            Self::OneshotReceiver(ref err) => Some(err),
            Self::BroadcastChannel(ref err) => Some(err),
            Self::ProcessorChannel(ref err) => Some(err),
            Self::CoreChannel(ref err) => Some(err),
        }
    }
}

impl From<common::errors::AppError> for SentinelError {
    fn from(err: common::errors::AppError) -> Self {
        match err {
            common::AppError::NoParentError(e) => Self::NoParent(e),
            common::AppError::BlockAlreadyInDbError(e) => Self::BlockAlreadyInDb(e),
            _ => Self::Common(err),
        }
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

impl From<tokio::sync::broadcast::error::SendError<BroadcasterMessages>> for SentinelError {
    fn from(err: tokio::sync::broadcast::error::SendError<BroadcasterMessages>) -> Self {
        Self::BroadcastChannel(Box::new(err))
    }
}

impl From<tokio::sync::mpsc::error::SendError<CoreMessages>> for SentinelError {
    fn from(err: tokio::sync::mpsc::error::SendError<CoreMessages>) -> Self {
        Self::CoreChannel(Box::new(err))
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

impl From<common_rocksdb::RocksdbDatabaseError> for SentinelError {
    fn from(err: common_rocksdb::RocksdbDatabaseError) -> Self {
        Self::RocksDb(err)
    }
}

impl<T> From<std::sync::PoisonError<T>> for SentinelError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::PoisonedLock
    }
}

impl From<std::io::Error> for SentinelError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<tokio::sync::mpsc::error::SendError<ProcessorMessages>> for SentinelError {
    fn from(err: tokio::sync::mpsc::error::SendError<ProcessorMessages>) -> Self {
        Self::ProcessorChannel(Box::new(err))
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for SentinelError {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::OneshotReceiver(err)
    }
}

impl From<std::time::SystemTimeError> for SentinelError {
    fn from(err: std::time::SystemTimeError) -> Self {
        Self::Time(err)
    }
}

impl From<tokio::sync::mpsc::error::SendError<MongoMessages>> for SentinelError {
    fn from(err: tokio::sync::mpsc::error::SendError<MongoMessages>) -> Self {
        Self::MongoChannel(Box::new(err))
    }
}

impl From<ethabi::Error> for SentinelError {
    fn from(err: ethabi::Error) -> Self {
        Self::EthAbi(err)
    }
}
