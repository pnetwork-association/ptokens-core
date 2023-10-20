use common::AppError as CommonError;
use common_chain_ids::EthChainId;
use common_metadata::MetadataChainIdError;
use thiserror::Error;

use crate::{
    BroadcastChannelMessages,
    ChallengeResponderMessages,
    DbKey,
    EthRpcMessages,
    NetworkId,
    StatusPublisherMessages,
    SyncerMessages,
    UserOpCancellerMessages,
    WebSocketMessages,
};

impl From<SentinelError> for CommonError {
    fn from(e: SentinelError) -> CommonError {
        CommonError::Custom(format!("{e}"))
    }
}

#[derive(Error, Debug)]
pub enum SentinelError {
    #[error("no nonce for network id {0}")]
    NoNonce(NetworkId),

    #[error("invalid frequency {frequency} - must be between {min} & {max}")]
    InvalidFrequency { min: u64, max: u64, frequency: u64 },

    #[error("{0}")]
    Challenges(#[from] crate::ChallengesError),

    #[error("{0}")]
    Actors(#[from] crate::ActorsError),

    #[error("{0}")]
    Ipfs(#[from] crate::IpfsError),

    #[error("{0}")]
    SentinelStatusError(#[from] crate::status::SentinelStatusError),

    #[error("chain error: {0}")]
    ChainError(#[from] common_eth::ChainError),

    #[error("{0}")]
    MetadataChainId(#[from] MetadataChainIdError),

    #[error("chain id not in config: {0}")]
    ChainIdNotInConfig(EthChainId),

    #[error("rustc hex error: {0}")]
    RustCHex(#[from] rustc_hex::FromHexError),

    #[error("no core connected to sentinel app")]
    NoCore,

    #[error("a java exception occurred and was handled - see core logs for details")]
    JavaExceptionOccurred,

    #[error("no latest block number for network ID: {0}")]
    NoLatestBlockNumber(NetworkId),

    #[error("timed out whilst {0}")]
    Timedout(String),

    #[error("websocket messages error: {0}")]
    WebSocketMessages(#[from] crate::messages::WebSocketMessagesError),

    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),

    #[error("tokio try lock error: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),

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

    #[error("poisoned lock encountered when accessing")]
    PoisonedLock(String),

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

    #[error("no parent error")]
    NoParent(common::NoParentError),

    #[error("system time error: {0}")]
    Time(#[from] std::time::SystemTimeError),

    #[error("batching error: {0}")]
    Batching(#[from] crate::batching::BatchingError),

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
    SentinelConfig(#[from] crate::config::SentinelConfigError),

    #[error("{0}")]
    BlockAlreadyInDb(common::BlockAlreadyInDbError),

    #[error("tokio receiver error: {0}")]
    Receiver(#[from] tokio::sync::broadcast::error::RecvError),

    #[error("tokio oneshot error: {0}")]
    OneshotReceiver(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("eth rpc channel error: {0}")]
    EthRpcChannel(Box<tokio::sync::mpsc::error::SendError<EthRpcMessages>>),

    #[error("websocket channel error: {0}")]
    WebSocketChannel(Box<tokio::sync::mpsc::error::SendError<WebSocketMessages>>),

    #[error("status channel error: {0}")]
    StatusChannel(Box<tokio::sync::mpsc::error::SendError<StatusPublisherMessages>>),

    #[error("challenge responder channel error: {0}")]
    ChallengeResponderChannel(Box<tokio::sync::mpsc::error::SendError<ChallengeResponderMessages>>),

    #[error("syncer channel error: {0}")]
    SyncerChannel(Box<tokio::sync::broadcast::error::SendError<SyncerMessages>>),

    #[error("user op canceller channel error: {0}")]
    UserOpCancellerChannel(Box<tokio::sync::mpsc::error::SendError<UserOpCancellerMessages>>),

    #[error("broadcast messages channel error: {0}")]
    BroadcastChannelMessages(Box<tokio::sync::broadcast::error::SendError<BroadcastChannelMessages>>),
}

impl From<tokio::sync::broadcast::error::SendError<SyncerMessages>> for SentinelError {
    fn from(e: tokio::sync::broadcast::error::SendError<SyncerMessages>) -> Self {
        Self::SyncerChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<UserOpCancellerMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<UserOpCancellerMessages>) -> Self {
        Self::UserOpCancellerChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<EthRpcMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<EthRpcMessages>) -> Self {
        Self::EthRpcChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<WebSocketMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<WebSocketMessages>) -> Self {
        Self::WebSocketChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<StatusPublisherMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<StatusPublisherMessages>) -> Self {
        Self::StatusChannel(Box::new(e))
    }
}

impl From<tokio::sync::mpsc::error::SendError<ChallengeResponderMessages>> for SentinelError {
    fn from(e: tokio::sync::mpsc::error::SendError<ChallengeResponderMessages>) -> Self {
        Self::ChallengeResponderChannel(Box::new(e))
    }
}

impl From<tokio::sync::broadcast::error::SendError<BroadcastChannelMessages>> for SentinelError {
    fn from(e: tokio::sync::broadcast::error::SendError<BroadcastChannelMessages>) -> Self {
        Self::BroadcastChannelMessages(Box::new(e))
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
