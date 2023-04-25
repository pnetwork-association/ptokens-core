use ethereum_types::H256 as EthHash;
use thiserror::Error;

use super::UserOpState;

#[derive(Error, Debug)]
pub enum UserOpError {
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

    #[error("cannot cancel because user op was {0}")]
    CannotCancel(UserOpState),

    #[error("user ops UIDs do not match ({a} != {b})")]
    UidMismatch { a: EthHash, b: EthHash },
}