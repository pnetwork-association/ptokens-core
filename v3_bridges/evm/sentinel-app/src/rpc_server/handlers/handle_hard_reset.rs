use common_debug_signers::DebugSignature;
use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_hard_reset(
        params: RpcParams,
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let debug_sig = if params.is_empty() {
            DebugSignature::new(None)
        } else {
            DebugSignature::new(params.get(1).cloned())
        };

        let msg = WebSocketMessagesEncodable::HardReset(debug_sig);

        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }
}
