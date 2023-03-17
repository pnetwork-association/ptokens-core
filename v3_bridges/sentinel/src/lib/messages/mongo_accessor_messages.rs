use common::BridgeSide;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{NativeOutput, Responder, SentinelError};

#[derive(Debug)]
pub enum MongoAccessorMessages {
    //PutHost(HostOutput),
    PutNative(NativeOutput),
}
