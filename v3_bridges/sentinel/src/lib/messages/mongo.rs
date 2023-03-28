use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{HeartbeatsJson, HostOutput, NativeOutput, Output, Responder, SentinelError};

#[derive(Debug)]
pub enum MongoMessages {
    PutHost(HostOutput),
    PutNative(NativeOutput),
    GetOutput(Responder<Output>),
    PutHeartbeats(HeartbeatsJson),
    GetHeartbeats(Responder<HeartbeatsJson>),
}

impl MongoMessages {
    pub fn get_heartbeats_msg() -> (Self, Receiver<Result<HeartbeatsJson, SentinelError>>) {
        let (resp_tx, resp_rx) = oneshot::channel();
        (Self::GetHeartbeats(resp_tx), resp_rx)
    }

    pub fn get_output_msg() -> (Self, Receiver<Result<Output, SentinelError>>) {
        let (resp_tx, resp_rx) = oneshot::channel();
        (Self::GetOutput(resp_tx), resp_rx)
    }
}
