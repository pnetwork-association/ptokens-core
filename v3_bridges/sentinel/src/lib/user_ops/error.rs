use thiserror::Error;

use super::UserOpState;

#[derive(Error, Debug)]
pub enum UserOpError {
    #[error("cannot update user op state from: '{0}'")]
    CannotUpdate(UserOpState),

    #[error("cannot cancel user op state from: '{0}'")]
    CannotCancel(UserOpState),

    #[error("user op processing error: {0}")]
    Process(String),

    #[error("{0}")]
    Sentinel(#[from] crate::SentinelError),

    #[error("infallible error: {0}")]
    Infallible(#[from] std::convert::Infallible),
}
