use common::{AppError as CommonError, BridgeSide};
use common_chain_ids::EthChainId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::SentinelError;

#[derive(Error, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesError {
    #[error("core not initialized for chain id: {0}")]
    Uninitialized(EthChainId),

    #[error("core already initialized for chain id: {0}")]
    AlreadyInitialized(EthChainId),

    #[error("cannot create websocket message encodable from args: {0:?}")]
    CannotCreate(Vec<String>),

    #[error("cannot create websocket message encodable from {got} args, expected {expected}: {args:?}")]
    NotEnoughArgs {
        got: usize,
        expected: usize,
        args: Vec<String>,
    },

    #[error("could not parse u64 from {0}")]
    ParseInt(String),

    #[error("unrecognized chain id {0}")]
    UnrecognizedEthChainId(String),

    #[error("timed out - strongbox took longer than {0}ms to respond")]
    Timedout(u64),

    #[error("no {side} block found in {struct_name}")]
    NoBlock { side: BridgeSide, struct_name: String },

    #[error("common error: {0}")]
    CommonError(String),

    #[error("sentinel error: {0}")]
    SentinelError(String),

    #[error("java database error: {0}")]
    JavaDb(String),

    #[error("unhandled websocket message: {0}")]
    Unhandled(String),

    #[error("cannot convert from: {from} to: {to}")]
    CannotConvert { from: String, to: String },
}

impl From<CommonError> for WebSocketMessagesError {
    fn from(e: CommonError) -> Self {
        Self::CommonError(format!("{e}"))
    }
}

impl From<SentinelError> for WebSocketMessagesError {
    fn from(e: SentinelError) -> Self {
        Self::SentinelError(format!("{e}"))
    }
}
