use common::AppError as CommonError;
use common_chain_ids::EthChainId;
use common_eth::{ChainError, NoParentError};
use common_metadata::MetadataChainId;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::SentinelError;

#[derive(Clone, Error, Debug, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesError {
    #[error("need more than {num_args} args")]
    NeedMoreArgs { num_args: usize },

    #[error("max delta sanity check failed - got {got}s, but min is {min}s and max is {max}s")]
    MaxDelta { got: u64, max: u64, min: u64 },

    #[error("insufficient mcids  - got {got}, expected {expected}")]
    InsufficientMcids { got: usize, expected: usize },

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

    #[error("cannot parse metadata chain id from: '{0}'")]
    UnrecognizedChainId(String),

    #[error("unsupported chain id {0}")]
    Unsupported(MetadataChainId),

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

    #[error("block {num} with hash {hash} already in db for chain id {mcid}")]
    BlockAlreadyInDb {
        num: u64,
        hash: EthHash,
        mcid: MetadataChainId,
    },

    #[error("unexpected websocket response {0}")]
    UnexpectedResponse(String),

    #[error("expected Some(..) arg name {arg_name} in location {location}, but got None")]
    NoneError { arg_name: String, location: String },

    #[error("{0}")]
    ChainError(ChainError),

    #[error("{0}")]
    Custom(String),
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

impl From<ChainError> for WebSocketMessagesError {
    fn from(e: ChainError) -> Self {
        match e {
            ChainError::NoParent(e) => Self::NoParent(e),
            ChainError::BlockAlreadyInDb { num, mcid, hash } => Self::BlockAlreadyInDb { num, mcid, hash },
            _ => Self::ChainError(e),
        }
    }
}

impl From<hex::FromHexError> for WebSocketMessagesError {
    fn from(e: hex::FromHexError) -> Self {
        Self::Hex(format!("{e}"))
    }
}
