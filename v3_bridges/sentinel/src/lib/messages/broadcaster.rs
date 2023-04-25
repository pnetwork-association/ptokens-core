use tokio::sync::{oneshot, oneshot::Receiver};
use crate::{UserOp, CoreState, Responder, SentinelError, UserOps};

#[derive(Debug)]
pub enum BroadcasterMessages {
    CancelUserOps(UserOps),
}
