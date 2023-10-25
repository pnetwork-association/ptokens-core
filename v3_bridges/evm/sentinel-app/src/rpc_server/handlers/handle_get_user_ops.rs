use common_sentinel::{SentinelError, WebSocketMessages, WebSocketMessagesEncodable};
use tokio::time::{sleep, Duration};

use crate::{
    rpc_server::{RpcCall, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_get_user_ops(
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetUserOps);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT)) => {
                let m = "getting user ops";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
