use common::AppError as CommonError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("custom: {0}")]
    Custom(String),

    #[error("jni error: {0}")]
    Jni(#[from] jni::errors::Error),

    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
}

impl From<Error> for CommonError {
    fn from(e: Error) -> Self {
        CommonError::Custom(format!("{e}"))
    }
}
