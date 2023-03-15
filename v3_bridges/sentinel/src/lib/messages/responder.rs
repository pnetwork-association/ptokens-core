use std::result::Result;

use tokio::sync::oneshot;

use crate::SentinelError;

pub type Responder<T> = oneshot::Sender<Result<T, SentinelError>>;
