use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_purge_user_ops(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let epoch = 0; // TODO/FIXME Take an epoch as an argument and only purge prior epochs
        let msg = WebSocketMessagesEncodable::PurgeUserOps(epoch, params.get(0).into());
        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }
}
