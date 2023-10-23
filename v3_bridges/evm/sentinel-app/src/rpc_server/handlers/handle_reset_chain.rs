use common::BridgeSide;
use common_sentinel::{
    call_core,
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesResetChainArgs,
};

use crate::{
    rpc_server::{RpcCall, RpcParams, STRONGBOX_TIMEOUT_MS},
    type_aliases::{EthRpcTx, WebSocketTx},
};

impl RpcCall {
    pub(crate) async fn handle_reset_chain(
        config: SentinelConfig,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let mut args = WebSocketMessagesResetChainArgs::try_from(params)?;

        let network_id = *args.network_id();
        let side = if network_id == *config.host().network_id() {
            BridgeSide::Host
        } else if network_id == *config.native().network_id() {
            BridgeSide::Native
        } else {
            return Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::Unsupported(
                network_id,
            )));
        };

        let block_num = if let Some(n) = args.block_num() {
            *n
        } else {
            let (msg, responder) = EthRpcMessages::get_latest_block_num_msg(network_id);
            if side.is_host() {
                host_eth_rpc_tx.send(msg).await?;
            } else {
                native_eth_rpc_tx.send(msg).await?;
            };
            responder.await??
        };

        debug!("getting sub mat for block num {block_num} for cid {network_id}");

        let (eth_rpc_msg, responder) = EthRpcMessages::get_sub_mat_msg(network_id, block_num);
        if side.is_host() {
            host_eth_rpc_tx.send(eth_rpc_msg).await?;
        } else {
            native_eth_rpc_tx.send(eth_rpc_msg).await?;
        };
        let sub_mat = responder.await??;

        args.add_sub_mat(sub_mat);
        args.add_side(side);

        let msg = WebSocketMessagesEncodable::ResetChain(Box::new(args));
        call_core(STRONGBOX_TIMEOUT_MS, websocket_tx.clone(), msg).await
    }
}
