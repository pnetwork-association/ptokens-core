use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{HeartbeatsJson, Responder, SentinelError};

#[derive(Debug)]
pub enum MongoMessages {
    PutHeartbeats(HeartbeatsJson),
    GetHeartbeats(Responder<HeartbeatsJson>),
}

impl MongoMessages {
    pub fn get_heartbeats_msg() -> (Self, Receiver<Result<HeartbeatsJson, SentinelError>>) {
        let (resp_tx, resp_rx) = oneshot::channel();
        (Self::GetHeartbeats(resp_tx), resp_rx)
    }
}
