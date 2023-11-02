use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_delete(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling db delete...");
        Self::check_core_is_connected(core_cxn)?;
        let msg = WebSocketMessagesEncodable::try_from(Self::create_args("delete", params))?;
        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }

    pub(crate) async fn handle_get(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling db get...");
        Self::check_core_is_connected(core_cxn)?;
        let msg = WebSocketMessagesEncodable::try_from(Self::create_args("get", params))?;
        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }

    pub(crate) async fn handle_put(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling db put...");
        Self::check_core_is_connected(core_cxn)?;
        let msg = WebSocketMessagesEncodable::try_from(Self::create_args("put", params))?;
        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }
}
