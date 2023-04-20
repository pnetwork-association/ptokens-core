use thiserror::Error;

use crate::{
    BroadcasterMessages,
    CoreMessages,
    DbKey,
    EthRpcMessages,
    MongoMessages,
    ProcessorMessages,
    SyncerMessages,
};

#[derive(Error, Debug)]
pub enum SentinelError {
    #[error("{0}")]
    UserOp(Box<crate::user_ops::UserOpError>),

    #[error("key exists in db: {0}")]
    KeyExists(DbKey),

    #[error("{0}")]
    NetworkId(#[from] crate::network_id::NetworkIdError),

    #[error("poisoned lock encountered")]
    PoisonedLock,

    #[error("no block {0}")]
    NoBlock(u64),

    #[error("{0}")]
    Custom(String),

    #[error("sigint caught in component '{0}'")]
    SigInt(String),

    #[error("timeout error: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),

    #[error("syncer to restart from block {0}")]
    SyncerRestart(u64),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("eth abi error: {0}")]
    EthAbi(#[from] ethabi::Error),

    #[error("hex error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("{0}")]
    Json(serde_json::Value),

    #[error("common error: {0}")]
    Common(#[from] common::AppError),

    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("serde json error {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("mongo error: {0}")]
    MongoDb(#[from] mongodb::error::Error),

    #[error("no parent error")]
    NoParent(common::NoParentError),

    #[error("system time error: {0}")]
    Time(#[from] std::time::SystemTimeError),

    #[error("batching error: {0}")]
    Batching(#[from] crate::batching::Error),

    #[error("parse int error {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("tokio join error {0}")]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error("logger error: {0}")]
    Logger(#[from] flexi_logger::FlexiLoggerError),

    #[error("json rpc error: {0}")]
    JsonRpc(#[from] jsonrpsee::core::error::Error),

    #[error("endpoint error {0}")]
    Endpoint(#[from] crate::endpoints::EndpointError),

    #[error("sentinel config error {0}")]
    SentinelConfig(#[from] crate::config::ConfigError),

    #[error("rocksdb error {0}")]
    RocksDb(#[from] common_rocksdb::RocksdbDatabaseError),

    #[error("{0}")]
    BlockAlreadyInDb(common::BlockAlreadyInDbError),

    #[error("ethers coreencode packed error: {0}")]
    EncodePacked(#[from] ethers_core::abi::EncodePackedError),

    #[error("tokio receiver error: {0}")]
    Receiver(#[from] tokio::sync::broadcast::error::RecvError),

    #[error("tokio oneshot error: {0}")]
    OneshotReceiver(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("core channel error: {0}")]
    CoreChannel(Box<tokio::sync::mpsc::error::SendError<CoreMessages>>),

    #[error("mongo channel error: {0}")]
    MongoChannel(Box<tokio::sync::mpsc::error::SendError<MongoMessages>>),

    #[error("eth rpc channel error: {0}")]
    EthRpcChannel(Box<tokio::sync::mpsc::error::SendError<EthRpcMessages>>),

    #[error("syncer channel error: {0}")]
    SyncerChannel(Box<tokio::sync::broadcast::error::SendError<SyncerMessages>>),

    #[error("processor channel error: {0}")]
    ProcessorChannel(Box<tokio::sync::mpsc::error::SendError<ProcessorMessages>>),

    #[error("broadcaster channel error: {0}")]
    BroadcastChannel(Box<tokio::sync::broadcast::error::SendError<BroadcasterMessages>>),
}

impl From<tokio::sync::broadcast::error::SendError<BroadcasterMessages>> for SentinelError {
    fn from(e: tokio::sync::broadcast::error::SendError<BroadcasterMessages>) -> Self {
        Self::BroadcastChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<CoreMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<CoreMessages>) -> Self {
        Self::CoreChannel(Box::new(e))
    }
}

impl From<tokio::sync::broadcast::error::SendError<SyncerMessages>> for SentinelError {
    fn from(e: tokio::sync::broadcast::error::SendError<SyncerMessages>) -> Self {
        Self::SyncerChannel(Box::new(e))
    }
}
impl From<tokio::sync::mpsc::error::SendError<ProcessorMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<ProcessorMessages>) -> Self {
        Self::ProcessorChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<MongoMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<MongoMessages>) -> Self {
        Self::MongoChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<EthRpcMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<EthRpcMessages>) -> Self {
        Self::EthRpcChannel(Box::new(e))
    }
}

impl From<crate::user_ops::UserOpError> for SentinelError {
    fn from(e: crate::user_ops::UserOpError) -> Self {
        Self::UserOp(Box::new(e))
    }
}
