use common::BridgeSide;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{HeartbeatsJson, HostOutput, NativeOutput, Responder, SentinelError};

#[derive(Debug)]
pub enum MongoAccessorMessages {
    PutHost(HostOutput),
    PutNative(NativeOutput),
    PutHeartbeats(HeartbeatsJson),
}
