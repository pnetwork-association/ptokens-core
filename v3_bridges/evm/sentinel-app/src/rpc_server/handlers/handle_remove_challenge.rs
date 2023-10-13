use std::str::FromStr;

use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::H256 as EthHash;

use crate::{
    rpc_server::{type_aliases::RpcParams, RpcCall, STRONGBOX_TIMEOUT_MS},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_remove_challenge(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;
        let hash = EthHash::from_str(&checked_params[0])?;
        call_core(
            STRONGBOX_TIMEOUT_MS,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::RemoveChallenge(hash),
        )
        .await
    }
}
