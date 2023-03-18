use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{HeartbeatsJson, HostOutput, NativeOutput, Responder, SentinelError};

#[derive(Debug)]
pub enum MongoAccessorMessages {
    PutHost(HostOutput),
    PutNative(NativeOutput),
    PutHeartbeats(HeartbeatsJson),
    GetHeartbeats(Responder<HeartbeatsJson>),
}

impl MongoAccessorMessages {
    pub fn get_heartbeats_msg() -> (Self, Receiver<Result<HeartbeatsJson, SentinelError>>) {
        let (resp_tx, resp_rx) = oneshot::channel();
        (Self::GetHeartbeats(resp_tx), resp_rx)
    }
}
