use common_sentinel::{
    call_core,
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesInitArgs,
};

use crate::{
    rpc_server::{RpcCall, RpcParams, STRONGBOX_TIMEOUT_MS},
    type_aliases::{EthRpcTx, WebSocketTx},
};

impl RpcCall {
    pub(crate) async fn handle_init(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let mut args = WebSocketMessagesInitArgs::try_from(params)?;
        let network_id = *args.network_id();

        let h_nid = *config.host().network_id();
        let n_nid = *config.native().network_id();

        let use_native = if network_id == n_nid {
            Result::<bool, SentinelError>::Ok(true)
        } else if network_id == h_nid {
            Result::<bool, SentinelError>::Ok(false)
        } else {
            Err(WebSocketMessagesError::Unsupported(network_id).into())
        }?;

        // NOTE: Now we can get the latest block number for the correct chain from the ETH RPC
        let (latest_block_num_msg, latest_block_num_responder) = EthRpcMessages::get_latest_block_num_msg(network_id);

        if use_native {
            native_eth_rpc_tx.send(latest_block_num_msg).await?;
        } else {
            host_eth_rpc_tx.send(latest_block_num_msg).await?;
        };

        let latest_block_num = latest_block_num_responder.await??;

        // NOTE: Get use that number to get the latest submission material
        let (latest_block_msg, latest_block_responder) = EthRpcMessages::get_sub_mat_msg(network_id, latest_block_num);
        if use_native {
            native_eth_rpc_tx.send(latest_block_msg).await?;
        } else {
            host_eth_rpc_tx.send(latest_block_msg).await?;
        }
        let sub_mat = latest_block_responder.await??;

        // NOTE: Now we need to add the sub mat to the args to send to strongbox
        args.add_sub_mat(sub_mat);

        call_core(
            STRONGBOX_TIMEOUT_MS,
            websocket_tx.clone(),
            WebSocketMessagesEncodable::Initialize(Box::new(args)),
        )
        .await
    }
}
