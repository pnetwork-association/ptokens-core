use tokio::{
    sync::mpsc::Sender as MpscTx,
    time::{sleep, Duration},
};

use crate::{SentinelError, WebSocketMessages, WebSocketMessagesEncodable};

pub async fn call_core(
    timeout: u64,
    websocket_tx: MpscTx<WebSocketMessages>,
    encodable_msg: WebSocketMessagesEncodable,
) -> Result<WebSocketMessagesEncodable, SentinelError> {
    let err_msg = format!("timed out after {timeout}s whilst handling core message {encodable_msg}");

    let (msg, rx) = WebSocketMessages::new(encodable_msg);
    websocket_tx.send(msg).await?;

    tokio::select! {
        r = rx => r?,
        _ = sleep(Duration::from_secs(timeout)) => {
            error!("{err_msg}");
            Err(SentinelError::Timedout(err_msg))
        }
    }
}
