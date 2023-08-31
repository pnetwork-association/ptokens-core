use common::AppError as CommonError;
use thiserror::Error;

use crate::{BroadcasterMessages, CoreMessages, DbKey, EthRpcMessages, MongoMessages, SyncerMessages};

impl From<SentinelError> for CommonError {
    fn from(e: SentinelError) -> CommonError {
        CommonError::Custom(format!("{e}"))
    }
}

#[derive(Error, Debug)]
pub enum SentinelError {
    #[error("jni error: {0}")]
    JniError(#[from] jni::errors::Error),

    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("warp hyper: {0}")]
    WarpHyper(#[from] warp::hyper::Error),

    #[error("env error: {0}")]
    Env(#[from] crate::env::EnvError),

    #[error("dotenv error: {0}")]
    DotEnv(#[from] dotenv::Error),

    #[error("{0}")]
    FromStrRadix(#[from] ethereum_types::FromStrRadixErr),

    #[error("{0}")]
    UserOp(Box<crate::user_ops::UserOpError>),

    #[error("key exists in db: {0}")]
    KeyExists(DbKey),

    #[error("network ID error: {0}")]
    NetworkId(#[from] crate::network_id::NetworkIdError),

    #[error("poisoned lock encountered")]
    PoisonedLock,

    #[error("no block {0}")]
    NoBlock(u64),

    #[error("{0}")]
    Custom(String),

    #[error("sigint caught in {0}")]
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
    Common(common::AppError),

    #[error("config crate error: {0}")]
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
    SentinelConfig(#[from] crate::sentinel_config::SentinelConfigError),

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

    #[error("broadcast channel error: {0}")]
    BroadcastChannel(Box<tokio::sync::broadcast::error::SendError<BroadcasterMessages>>),

    #[error("broadcaster channel error: {0}")]
    BroadcasterChannel(Box<tokio::sync::mpsc::error::SendError<BroadcasterMessages>>),
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

impl From<tokio::sync::mpsc::error::SendError<MongoMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<MongoMessages>) -> Self {
        Self::MongoChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<BroadcasterMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<BroadcasterMessages>) -> Self {
        Self::BroadcasterChannel(Box::new(e))
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

impl From<common::AppError> for SentinelError {
    fn from(e: common::AppError) -> Self {
        match e {
            common::AppError::NoParentError(e) => Self::NoParent(e),
            common::AppError::BlockAlreadyInDbError(e) => Self::BlockAlreadyInDb(e),
            _ => Self::Common(e),
        }
    }
}
