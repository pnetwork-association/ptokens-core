use common::{AppError as CommonError, BlockAlreadyInDbError, NoParentError};
use common_chain_ids::EthChainId;
use common_metadata::MetadataChainId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::SentinelError;

#[derive(Error, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesError {
    #[error("wrong field of enum - got: {got}, expected {expected}")]
    WrongField { got: String, expected: String },

    #[error("could not parse metadata chain id from string: {0}")]
    ParseMetadataChainId(String),

    #[error("strongbox panicked - check the logs for more info")]
    Panicked,

    #[error("from hex error: {0}")]
    Hex(String),

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

    #[error("unsupported chain id {0}")]
    Unsupported(EthChainId),

    #[error("timed out - strongbox took longer than {0}ms to respond")]
    Timedout(u64),

    #[error("no block found in {struct_name} for chain: {mcid}")]
    NoBlock { mcid: MetadataChainId, struct_name: String },

    #[error("common error: {0}")]
    CommonError(String),

    #[error("sentinel error: {0}")]
    SentinelError(String),

    #[error("java database error: {0}")]
    JavaDb(String),

    #[error("unhandled websocket message: {0}")]
    Unhandled(String),

    #[error("cannot convert from: '{from}' to: '{to}'")]
    CannotConvert { from: String, to: String },

    #[error("{0}")]
    NoParent(NoParentError),

    #[error("{0}")]
    BlockAlreadyInDb(BlockAlreadyInDbError),

    #[error("unexpected websocket response {0}")]
    UnexpectedResponse(String),

    #[error("expected Some(..) arg name {arg_name} in location {location}, but got None")]
    NoneError { arg_name: String, location: String },
}

impl From<CommonError> for WebSocketMessagesError {
    fn from(e: CommonError) -> Self {
        Self::CommonError(format!("{e}"))
    }
}

impl From<SentinelError> for WebSocketMessagesError {
    fn from(e: SentinelError) -> Self {
        match e {
            SentinelError::NoParent(e) => Self::NoParent(e),
            SentinelError::BlockAlreadyInDb(e) => Self::BlockAlreadyInDb(e),
            err => Self::SentinelError(format!("{err}")),
        }
    }
}

impl From<hex::FromHexError> for WebSocketMessagesError {
    fn from(e: hex::FromHexError) -> Self {
        Self::Hex(format!("{e}"))
    }
}
