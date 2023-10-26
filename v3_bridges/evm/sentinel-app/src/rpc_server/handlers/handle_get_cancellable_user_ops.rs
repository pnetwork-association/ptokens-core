use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_cancellable_user_ops(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let msg = WebSocketMessagesEncodable::try_from(Self::create_args("getCancellableUserOps", params))?;
        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }
}
