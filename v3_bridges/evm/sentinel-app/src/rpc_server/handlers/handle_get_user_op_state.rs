use std::str::FromStr;

use common_sentinel::{
    EthRpcMessages,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    UserOpUniqueId,
    UserOps,
    WebSocketMessagesEncodable,
};
use serde_json::json;

use crate::{
    rpc_server::{RpcCalls, RpcParams},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_get_user_op_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        eth_rpc_senders: EthRpcSenders,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling get user op state...");
        let checked_params = Self::check_params(params, 1)?;
        let uid = UserOpUniqueId::from_str(&checked_params[0])?;

        // NOTE: Core cxn checked for us in list handler
        let user_ops = UserOps::try_from(Self::handle_get_user_ops(websocket_tx, core_cxn).await?)?;

        let user_op = user_ops.get(&uid)?;

        let o_id = *user_op.origin_network_id();
        let d_id = user_op.destination_network_id();

        let (o_msg, o_rx) = EthRpcMessages::get_user_op_state_msg(o_id, user_op.clone(), config.pnetwork_hub(&o_id)?);

        let (d_msg, d_rx) = EthRpcMessages::get_user_op_state_msg(d_id, user_op, config.pnetwork_hub(&d_id)?);

        eth_rpc_senders.sender(&o_id)?.send(o_msg).await?;
        eth_rpc_senders.sender(&d_id)?.send(d_msg).await?;

        let origin_user_op_state = o_rx.await??;
        let destination_user_op_state = d_rx.await??;

        Ok(WebSocketMessagesEncodable::Success(json!({
            "uid": uid,
            "origigNetworkId": o_id,
            "originState": origin_user_op_state.to_string(),
            "destinationNetworkId": d_id,
            "destinationState": destination_user_op_state.to_string(),
        })))
    }
}
