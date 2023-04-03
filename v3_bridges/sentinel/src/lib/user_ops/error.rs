use thiserror::Error;

use super::UserOpState;

#[derive(Error, Debug)]
pub enum UserOpError {
    #[error("cannot update user op state from: '{0}'")]
    CannotUpdate(UserOpState),

    #[error("cannot cancel user op state from: '{0}'")]
    CannotCancel(UserOpState),
}
