use common_sentinel::{call_core, NetworkId, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCall, RpcParams, STRONGBOX_TIMEOUT_MS},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_get_latest_block_numbers(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let network_ids = params
            .iter()
            .map(|s| NetworkId::try_from(s).map_err(|e| e.into()))
            .collect::<Result<Vec<NetworkId>, SentinelError>>()?;

        call_core(
            STRONGBOX_TIMEOUT_MS,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetLatestBlockNumbers(network_ids),
        )
        .await
    }
}
