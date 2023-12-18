use common_sentinel::{call_core, SentinelConfig, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_cancellable_user_ops(
        websocket_tx: WebSocketTx,
        config: Box<SentinelConfig>,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        call_core(
            STRONGBOX_TIMEOUT,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetCancellableUserOps(config),
        )
        .await
    }
}
