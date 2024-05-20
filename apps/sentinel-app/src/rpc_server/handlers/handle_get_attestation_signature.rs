use common::strip_hex_prefix;
use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_attestation_signature(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling get attestation rpc call...");
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;

        let bytes = hex::decode(strip_hex_prefix(&checked_params[0])).map_err(|e| {
            error!("{e}");
            SentinelError::Custom(format!("invalid hex: '{}'", checked_params[0]))
        })?;

        call_core(
            STRONGBOX_TIMEOUT,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetAttestationSignature(bytes),
        )
        .await
    }
}
