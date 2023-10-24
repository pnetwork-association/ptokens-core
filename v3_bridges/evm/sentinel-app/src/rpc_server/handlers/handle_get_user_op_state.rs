use std::str::FromStr;

use common_sentinel::{
    EthRpcMessages,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    UserOpUniqueId,
    UserOps,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use serde_json::json;

use crate::{
    rpc_server::{RpcCall, RpcParams},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_get_user_op_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        eth_rpc_senders: EthRpcSenders,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        todo!("this");
        /*
        debug!("handling get user op state...");
        let checked_params = Self::check_params(params, 1)?;
        let uid = UserOpUniqueId::from_str(&checked_params[0])?;

        // NOTE: Core cxn checked for us in list handler
        let user_ops = match Self::handle_get_user_ops(websocket_tx, core_cxn).await? {
            WebSocketMessagesEncodable::Success(j) => {
                Ok::<UserOps, SentinelError>(serde_json::from_value::<UserOps>(j)?)
            },
            WebSocketMessagesEncodable::Error(e) => Err(e.into()),
            other => Err(WebSocketMessagesError::UnexpectedResponse(other.to_string()).into()),
        }?;

        let user_op = user_ops.get(&uid)?;
        let origin_network_id = *user_op.origin_network_id();
        let destination_network_id = user_op.destination_network_id();

        let (origin_msg, origin_rx) = EthRpcMessages::get_user_op_state_msg(
            origin_network_id,
            user_op.clone(),
            config.pnetwork_hub(&origin_network_id)?,
        );

        let (destination_msg, destination_rx) = EthRpcMessages::get_user_op_state_msg(
            destination_network_id,
            user_op,
            config.pnetwork_hub(&destination_network_id)?,
        );

        if destination_network_id == *config.host().network_id() {
            native_eth_rpc_tx.send(origin_msg).await?;
            host_eth_rpc_tx.send(destination_msg).await?;
        } else {
            host_eth_rpc_tx.send(origin_msg).await?;
            native_eth_rpc_tx.send(destination_msg).await?;
        };

        let origin_user_op_state = origin_rx.await??;
        let destination_user_op_state = destination_rx.await??;

        Ok(WebSocketMessagesEncodable::Success(json!({
            "uid": uid,
            "origigNetworkId": origin_network_id,
            "originState": origin_user_op_state.to_string(),
            "destinationNetworkId": destination_network_id,
            "destinationState": destination_user_op_state.to_string(),
        })))
        */
    }
}
