use tokio::sync::{oneshot, oneshot::Receiver};

use super::WebSocketMessagesEncodable;
use crate::{Responder, SentinelError};

#[derive(Debug)]
pub struct WebSocketMessages(
    pub WebSocketMessagesEncodable,
    pub Responder<WebSocketMessagesEncodable>,
);

impl WebSocketMessages {
    pub fn new(msg: WebSocketMessagesEncodable) -> (Self, Receiver<Result<WebSocketMessagesEncodable, SentinelError>>) {
        let (tx, rx) = oneshot::channel();
        (Self(msg, tx), rx)
    }
}
