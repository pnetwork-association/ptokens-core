use common::BridgeSide;
use common_sentinel::{EthRpcMessages, SentinelError, WebSocketMessages, WebSocketMessagesEncodable};
use tokio::time::{sleep, Duration};

use crate::rpc_server::{
    constants::{EthRpcTx, RpcParams, WebSocketTx, STRONGBOX_TIMEOUT_MS},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_init(
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        // NOTE: Get the latest host & native block numbers from the ETH RPC
        let (host_latest_block_num_msg, host_latest_block_num_responder) =
            EthRpcMessages::get_latest_block_num_msg(BridgeSide::Host);
        let (native_latest_block_num_msg, native_latest_block_num_responder) =
            EthRpcMessages::get_latest_block_num_msg(BridgeSide::Native);
        host_eth_rpc_tx.send(host_latest_block_num_msg).await?;
        native_eth_rpc_tx.send(native_latest_block_num_msg).await?;
        let host_latest_block_num = host_latest_block_num_responder.await??;
        let native_latest_block_num = native_latest_block_num_responder.await??;

        // NOTE: Get submission material for those latest block numbers
        let (host_latest_block_msg, host_latest_block_responder) =
            EthRpcMessages::get_sub_mat_msg(BridgeSide::Host, host_latest_block_num);
        let (native_latest_block_msg, native_latest_block_responder) =
            EthRpcMessages::get_sub_mat_msg(BridgeSide::Native, native_latest_block_num);
        host_eth_rpc_tx.send(host_latest_block_msg).await?;
        native_eth_rpc_tx.send(native_latest_block_msg).await?;
        let host_sub_mat = host_latest_block_responder.await??;
        let native_sub_mat = native_latest_block_responder.await??;

        let encodable_msg = WebSocketMessagesEncodable::try_from(Self::create_args("init", params))?;

        // NOTE: Now we need to add the sub mat to the args to send to strongbox
        let final_msg = match encodable_msg {
            WebSocketMessagesEncodable::Initialize(mut args) => {
                args.add_host_block(host_sub_mat);
                args.add_native_block(native_sub_mat);
                Ok(WebSocketMessagesEncodable::Initialize(args))
            },
            _ => Err(SentinelError::Custom("failed to crate initialize arguments".into())),
        }?;

        // NOTE: Now we send out msg to the websocket loop
        let (msg, rx) = WebSocketMessages::new(final_msg);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "initializing core";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
