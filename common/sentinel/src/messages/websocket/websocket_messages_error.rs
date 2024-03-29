use common::AppError as CommonError;
use common_eth::{ChainError, NoParentError};
use common_network_ids::NetworkId;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use thiserror::Error;

use crate::SentinelError;

#[derive(Clone, Error, Debug, PartialEq, Serialize, Deserialize)]
pub enum WebSocketMessagesError {
    #[error("{0} core not initialized")]
    NotInitialized(NetworkId),

    #[error("no inclusion proof in encryted database")]
    NoInclusionProof,

    #[error("need more than {num_args} args")]
    NeedMoreArgs { num_args: usize },

    #[error("max delta sanity check failed - got {got}s, but min is {min}s and max is {max}s")]
    MaxDelta { got: u64, max: u64, min: u64 },

    #[error("insufficient mcids  - got {got}, expected {expected}")]
    InsufficientMcids { got: usize, expected: usize },

    #[error("wrong field of enum - got: {got}, expected {expected}")]
    WrongField { got: String, expected: String },

    #[error("could not parse network id from string: {0}")]
    ParseNetworkId(String),

    #[error("strongbox panicked - check the logs for more info")]
    Panicked,

    #[error("from hex error: {0}")]
    Hex(String),

    #[error("cannot create websocket message encodable from args: {0:?}")]
    CannotCreate(Vec<String>),

    #[error("cannot create websocket message encodable from {got} args, expected at least {expected}: {args:?}")]
    NotEnoughArgs {
        got: usize,
        expected: usize,
        args: Vec<String>,
    },

    #[error("could not parse u64 from {0}")]
    ParseInt(String),

    #[error("cannot parse network id from: '{0}'")]
    UnrecognizedNetworkId(String),

    #[error("unsupported network id {0}")]
    Unsupported(NetworkId),

    #[error("timed out - strongbox took longer than {0}ms to respond")]
    Timedout(u64),

    #[error("no block found in {struct_name} for chain: {network_id}")]
    NoBlock { network_id: NetworkId, struct_name: String },

    #[error("common error: {0}")]
    CommonError(String),

    #[error("sentinel error: {0}")]
    SentinelError(String),

    #[error("{0}")]
    Json(Json),

    #[error("java database error: {0}")]
    JavaDb(String),

    #[error("unhandled websocket message: {0}")]
    Unhandled(String),

    #[error("cannot convert from: '{from}' to: '{to}'")]
    CannotConvert { from: String, to: String },

    #[error("{0}")]
    NoParent(NoParentError),

    #[error("block {num} with hash {hash} already in db for network id {network_id}")]
    BlockAlreadyInDb {
        num: u64,
        hash: EthHash,
        network_id: NetworkId,
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
        match e {
            CommonError::Json(j) => Self::Json(j),
            other => Self::CommonError(format!("{other}")),
        }
    }
}

impl From<SentinelError> for WebSocketMessagesError {
    fn from(e: SentinelError) -> Self {
        match e {
            SentinelError::Json(j) => Self::Json(j),
            other => Self::SentinelError(format!("{other}")),
        }
    }
}

impl From<ChainError> for WebSocketMessagesError {
    fn from(e: ChainError) -> Self {
        match e {
            ChainError::NoParent(e) => Self::NoParent(e),
            ChainError::BlockAlreadyInDb { num, mcid, hash } => Self::BlockAlreadyInDb {
                num,
                network_id: NetworkId::try_from(mcid).expect("mcid -> nid conversion not to fail"),
                hash,
            },
            _ => Self::ChainError(e),
        }
    }
}

impl From<hex::FromHexError> for WebSocketMessagesError {
    fn from(e: hex::FromHexError) -> Self {
        Self::Hex(format!("{e}"))
    }
}
