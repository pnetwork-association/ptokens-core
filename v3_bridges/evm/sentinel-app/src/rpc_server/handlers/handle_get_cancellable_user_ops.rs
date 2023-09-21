use common_sentinel::{SentinelConfig, SentinelError, WebSocketMessages, WebSocketMessagesEncodable};
use tokio::time::{sleep, Duration};

use crate::rpc_server::{
    constants::{WebSocketTx, STRONGBOX_TIMEOUT_MS},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_get_cancellable_user_ops(
        config: Box<SentinelConfig>,
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let max_delta = config.core().max_cancellable_time_delta();
        let encodable_msg = WebSocketMessagesEncodable::GetCancellableUserOps(*max_delta);
        let (msg, rx) = WebSocketMessages::new(encodable_msg);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "getting cancellable user ops";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
