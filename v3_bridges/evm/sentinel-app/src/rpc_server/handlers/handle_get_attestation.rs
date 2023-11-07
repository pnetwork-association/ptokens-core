use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_attestation(
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling get attestation rpc call...");
        Self::check_core_is_connected(core_cxn)?;
        call_core(
            STRONGBOX_TIMEOUT,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetAttestation,
        )
        .await
    }
}
