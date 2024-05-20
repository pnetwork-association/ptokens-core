use common_sentinel::{
    call_core,
    EthRpcMessages,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesInitArgs,
};

use crate::{
    rpc_server::{RpcCalls, RpcParams, STRONGBOX_TIMEOUT},
    type_aliases::WebSocketTx,
};

impl RpcCalls {
    pub(crate) async fn handle_init(
        _config: SentinelConfig,
        websocket_tx: WebSocketTx,
        eth_rpc_senders: EthRpcSenders,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let mut args = WebSocketMessagesInitArgs::try_from(params)?;
        let network_id = *args.network_id();

        match eth_rpc_senders.sender(&network_id) {
            Err(e) => {
                error!("{e}");
                Err(WebSocketMessagesError::Unsupported(network_id).into())
            },
            Ok(sender) => {
                // NOTE: Get the latest block num from the RPC
                let (n_msg, n_rx) = EthRpcMessages::get_latest_block_num_msg(network_id);
                sender.send(n_msg).await?;
                let latest_block_num = n_rx.await??;

                // NOTE: Now use that number to get the latest submission material
                let (b_msg, b_rx) = EthRpcMessages::get_sub_mat_msg(network_id, latest_block_num);
                sender.send(b_msg).await?;
                let sub_mat = b_rx.await??;

                // NOTE: Now we need to add the sub mat to the args to send to strongbox
                args.add_sub_mat(sub_mat);

                call_core(
                    STRONGBOX_TIMEOUT,
                    websocket_tx.clone(),
                    WebSocketMessagesEncodable::Initialize(Box::new(args)),
                )
                .await
            },
        }
    }
}
