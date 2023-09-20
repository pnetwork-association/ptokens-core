use common::BridgeSide;
use common_sentinel::{
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesResetChainArgs,
};
use tokio::time::{sleep, Duration};

use crate::rpc_server::{
    constants::{EthRpcTx, RpcParams, WebSocketTx, STRONGBOX_TIMEOUT_MS},
    RpcCall,
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
        let mut args = WebSocketMessagesResetChainArgs::try_from(Self::create_args("reset", params))?;

        let side = if args.chain_id() == &config.chain_id(&BridgeSide::Host) {
            BridgeSide::Host
        } else if args.chain_id() == &config.chain_id(&BridgeSide::Native) {
            BridgeSide::Native
        } else {
            return Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::Unsupported(
                args.chain_id().clone(),
            )));
        };

        let block_num = if let Some(n) = args.block_num() {
            *n
        } else {
            let (msg, responder) = EthRpcMessages::get_latest_block_num_msg(side);
            if side.is_host() {
                host_eth_rpc_tx.send(msg).await?;
            } else {
                native_eth_rpc_tx.send(msg).await?;
            };
            responder.await??
        };

        debug!(
            "getting sub mat for block num {block_num} on side {side} for cid {}",
            args.chain_id()
        );

        let (eth_rpc_msg, responder) = EthRpcMessages::get_sub_mat_msg(side, block_num);
        if side.is_host() {
            host_eth_rpc_tx.send(eth_rpc_msg).await?;
        } else {
            native_eth_rpc_tx.send(eth_rpc_msg).await?;
        };
        let sub_mat = responder.await??;

        args.add_sub_mat(sub_mat);
        args.add_side(side);

        let encodable_msg = WebSocketMessagesEncodable::ResetChain(Box::new(args));

        let (websocket_msg, rx) = WebSocketMessages::new(encodable_msg);
        websocket_tx.send(websocket_msg).await?;
        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "submitting block";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
