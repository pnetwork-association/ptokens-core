use common_sentinel::{call_core, NetworkId, SentinelError, WebSocketMessagesEncodable};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_latest_block_infos(
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
            STRONGBOX_TIMEOUT,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetLatestBlockInfos(network_ids),
        )
        .await
    }
}
