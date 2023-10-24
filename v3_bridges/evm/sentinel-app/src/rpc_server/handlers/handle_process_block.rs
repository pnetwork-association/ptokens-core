use common_eth::EthSubmissionMaterials;
use common_sentinel::{
    call_core,
    EthRpcMessages,
    NetworkId,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesProcessBatchArgs,
};

use crate::{
    rpc_server::{RpcCall, RpcParams, STRONGBOX_TIMEOUT_MS},
    type_aliases::{EthRpcTx, WebSocketTx},
};

impl RpcCall {
    pub(crate) async fn handle_process_block(
        config: SentinelConfig,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 4)?;

        let network_id = NetworkId::try_from(&checked_params[0])?;
        let block_num = checked_params[1].parse::<u64>()?;
        let (eth_rpc_msg, responder) = EthRpcMessages::get_sub_mat_msg(network_id, block_num);
        //
        // FIXME We still need the "side" sometimes...
        let use_native = &network_id == config.native().network_id();

        if use_native {
            native_eth_rpc_tx.send(eth_rpc_msg).await?;
        } else {
            host_eth_rpc_tx.send(eth_rpc_msg).await?;
        };
        let sub_mat = responder.await??;

        let dry_run = matches!(checked_params[2].as_ref(), "true");
        let reprocess = matches!(checked_params[3].as_ref(), "true");

        let submit_args = WebSocketMessagesProcessBatchArgs::new(
            config.validate(&network_id)?,
            dry_run,
            reprocess,
            network_id,
            config.pnetwork_hub(&network_id)?,
            EthSubmissionMaterials::new(vec![sub_mat]), // NOTE: The processor always deals with batches of submat
            config.governance_address(&network_id),
        );
        let msg = WebSocketMessagesEncodable::ProcessBatch(Box::new(submit_args));
        call_core(STRONGBOX_TIMEOUT_MS, websocket_tx.clone(), msg).await
    }
}
