use common::Byte;
use ethabi::Token as EthAbiToken;
use ethereum_types::{H256 as EthHash, U256};
use thiserror::Error;

use super::{UserOp, UserOpState};

#[derive(Error, Debug)]
pub enum UserOpError {
    #[error("ethabi error: {0}")]
    EthAbi(#[from] ethabi::Error),

    #[error("not enough tokens, got: {got}, expected: {expected} in {location}")]
    NotEnoughTokens {
        got: usize,
        expected: usize,
        location: String,
    },

    #[error("cannot cancel user op from state: {0}")]
    CannotCancelOpInState(UserOpState),

    #[error("cannot determine smart-contract user op state from: {0}")]
    CannotDetermineUserOpSmartContractState(UserOpState),

    #[error("insufficient ETH balance to cancel tx - have: {have} need: {need}")]
    InsufficientBalance { have: U256, need: U256 },

    #[error("cannot update user op state from: '{from}' to {to}")]
    CannotUpdate { from: UserOpState, to: UserOpState },

    #[error("user op processing error: {0}")]
    Process(String),

    #[error("{0}")]
    Sentinel(#[from] crate::SentinelError),

    #[error("infallible error: {0}")]
    Infallible(#[from] std::convert::Infallible),

    #[error("no topics in log")]
    NoTopics,

    #[error("unrecognized topic hash: {0}")]
    UnrecognizedTopic(EthHash),

    #[error("{0}")]
    AppError(#[from] common::AppError),

    #[error("cannot cancel user op: {0}")]
    CannotCancel(Box<UserOp>),

    #[error("user ops UIDs do not match ({a} != {b})")]
    UidMismatch { a: EthHash, b: EthHash },

    #[error("unrecognized smart-contract user op state: {0}")]
    UnrecognizedSmartContractUserOpState(Byte),

    #[error("not enough bytes - got: {got}, expected: {expected} in '{location}'")]
    NotEnoughBytes {
        got: usize,
        expected: String,
        location: String,
    },

    #[error("`UserOpLog` is missing field: '{0}'")]
    MissingField(String),

    #[error("user op has not been enqueued")]
    HasNotBeenEnqueued,

    #[error("no user op exists with hash: {0}")]
    NoUserOp(EthHash),

    #[error("cannot convert ethabi token from: {from} to: {to}")]
    CannotConvertEthAbiToken { from: EthAbiToken, to: String },
}
