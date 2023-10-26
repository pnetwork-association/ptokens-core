use std::str::FromStr;

use common_sentinel::{SentinelError, UserOpUniqueId, WebSocketMessages, WebSocketMessagesEncodable};
use tokio::time::{sleep, Duration};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_user_op(
        params: RpcParams,
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;
        let h = UserOpUniqueId::from_str(&checked_params[0])?;
        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetUserOp(h));
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
