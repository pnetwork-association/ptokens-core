use std::str::FromStr;

use common_sentinel::{
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    UserOpUniqueId,
    UserOps,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use serde_json::json;

use crate::rpc_server::{
    constants::{EthRpcTx, RpcParams, WebSocketTx},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_get_user_op_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
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
        let origin_side = user_op.destination_side();
        let destination_side = user_op.destination_side();

        let (origin_msg, origin_rx) =
            EthRpcMessages::get_user_op_state_msg(origin_side, user_op.clone(), config.pnetwork_hub(&origin_side));
        let (destination_msg, destination_rx) =
            EthRpcMessages::get_user_op_state_msg(destination_side, user_op, config.pnetwork_hub(&destination_side));

        if destination_side.is_host() {
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
            "originChainId": config.chain_id(&origin_side),
            "originState": origin_user_op_state.to_string(),
            "destinationChainId": config.chain_id(&destination_side),
            "destinationState": destination_user_op_state.to_string(),
        })))
    }
}