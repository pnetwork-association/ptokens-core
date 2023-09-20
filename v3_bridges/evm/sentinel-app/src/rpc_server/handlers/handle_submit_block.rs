use std::str::FromStr;

use common::BridgeSide;
use common_eth::EthSubmissionMaterials;
use common_sentinel::{
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesSubmitArgs,
};
use tokio::time::{sleep, Duration};

use crate::rpc_server::{
    constants::{EthRpcTx, RpcParams, WebSocketTx, STRONGBOX_TIMEOUT_MS},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_submit_block(
        config: SentinelConfig,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 4)?;

        let side = BridgeSide::from_str(&checked_params[0])?;
        let block_num = checked_params[1].parse::<u64>()?;
        let (eth_rpc_msg, responder) = EthRpcMessages::get_sub_mat_msg(side, block_num);
        if side.is_host() {
            host_eth_rpc_tx.send(eth_rpc_msg).await?;
        } else {
            native_eth_rpc_tx.send(eth_rpc_msg).await?;
        };
        let sub_mat = responder.await??;

        let dry_run = matches!(checked_params[2].as_ref(), "true");
        let reprocess = matches!(checked_params[3].as_ref(), "true");

        let submit_args = WebSocketMessagesSubmitArgs::new(
            dry_run,
            config.is_validating(&side),
            reprocess,
            side,
            config.chain_id(&side),
            config.pnetwork_hub(&side),
            EthSubmissionMaterials::new(vec![sub_mat]), // NOTE: The processor always deals with batches of submat
        );
        let encodable_msg = WebSocketMessagesEncodable::Submit(Box::new(submit_args));

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
