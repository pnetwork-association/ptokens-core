use common_sentinel::{call_core, NetworkId, SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError};

use crate::{
    rpc_server::{RpcCall, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_get_status(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let n = 1;
        let l = params.len();
        if l < n {
            return Err(WebSocketMessagesError::NeedMoreArgs { num_args: n }.into());
        }

        let network_ids = params
            .iter()
            .map(|s| NetworkId::try_from(s).map_err(|_| WebSocketMessagesError::ParseNetworkId(s.into())))
            .collect::<Result<Vec<NetworkId>, WebSocketMessagesError>>()?;

        call_core(
            STRONGBOX_TIMEOUT,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetStatus(network_ids),
        )
        .await
    }
}
