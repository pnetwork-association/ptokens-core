use common::BridgeSide;
use common_sentinel::{
    call_core,
    EthRpcMessages,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesResetChainArgs,
};

use crate::{
    rpc_server::{RpcCall, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCall {
    pub(crate) async fn handle_reset_chain(
        config: SentinelConfig,
        eth_rpc_senders: EthRpcSenders,
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let mut args = WebSocketMessagesResetChainArgs::try_from(params)?;
        let network_id = *args.network_id();

        match eth_rpc_senders.sender(&network_id) {
            Err(e) => {
                error!("{e}");
                Err(WebSocketMessagesError::Unsupported(network_id).into())
            },
            Ok(sender) => {
                let block_num = if let Some(n) = args.block_num() {
                    *n
                } else {
                    let (msg, rx) = EthRpcMessages::get_latest_block_num_msg(network_id);
                    sender.send(msg).await?;
                    rx.await??
                };

                debug!("getting sub mat for block num {block_num} for cid {network_id}");

                let (eth_rpc_msg, responder) = EthRpcMessages::get_sub_mat_msg(network_id, block_num);
                sender.send(eth_rpc_msg).await?;
                let sub_mat = responder.await??;
                args.add_sub_mat(sub_mat);
                let msg = WebSocketMessagesEncodable::ResetChain(Box::new(args));
                call_core(STRONGBOX_TIMEOUT, websocket_tx.clone(), msg).await
            },
        }
    }
}
