use common_sentinel::{
    Challenge,
    EthRpcMessages,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
};
use serde_json::json;

use crate::{
    rpc_server::{RpcCall, RpcParams},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_get_challenge_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        eth_rpc_senders: EthRpcSenders,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        todo!("this");
        /*
        debug!("handling get challenge state...");
        let checked_params = Self::check_params(params, 1)?;

        // NOTE: Core cxn checked for us in list handler
        let challenge = Challenge::try_from(Self::handle_get_challenge(websocket_tx, checked_params, core_cxn).await?)?;

        let network_id = challenge.network_id();

        // NOTE We're still stuck with host and native for now, so we need to figure out which of
        // those this challenge originated.
        let (msg, rx) =
            EthRpcMessages::get_challenge_state_msg(*network_id, challenge, config.pnetwork_hub(network_id)?);

        if config.native().network_id() == network_id {
            warn!("using bridge side NATIVE");
            native_eth_rpc_tx.send(msg).await?;
        } else {
            warn!("using bridge side HOST");
            host_eth_rpc_tx.send(msg).await?;
        };

        let state = rx.await??;

        Ok(WebSocketMessagesEncodable::Success(json!(state)))
        */
    }
}