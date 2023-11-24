use common_debug_signers::DebugSignature;
use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_remove_debug_signer(
        params: RpcParams,
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;

        // NOTE: We expect the signer to be the eth address in hex format
        let signer = checked_params[0].clone();

        let sig = if checked_params.len() > 1 {
            DebugSignature::new(checked_params.get(1).cloned())
        } else {
            DebugSignature::new(None)
        };

        let msg = WebSocketMessagesEncodable::RemoveDebugSigner(signer, sig);

        call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
    }
}
