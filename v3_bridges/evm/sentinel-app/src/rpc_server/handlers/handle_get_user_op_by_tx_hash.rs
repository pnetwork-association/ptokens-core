use std::str::FromStr;

use common_sentinel::{call_core, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::H256 as EthHash;

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_user_op_by_tx_hash(
        params: RpcParams,
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;
        let h = EthHash::from_str(&checked_params[0])?;
        call_core(
            STRONGBOX_TIMEOUT,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::GetUserOpByTxHash(h),
        )
        .await
    }
}
