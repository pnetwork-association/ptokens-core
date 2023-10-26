use common_sentinel::{SentinelError, WebSocketMessages, WebSocketMessagesEncodable};
use tokio::time::{sleep, Duration};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_remove_user_op(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let encodable_msg = WebSocketMessagesEncodable::try_from(Self::create_args("removeUserOp", params))?;
        let (msg, rx) = WebSocketMessages::new(encodable_msg);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT)) => {
                let m = "removing user op";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
