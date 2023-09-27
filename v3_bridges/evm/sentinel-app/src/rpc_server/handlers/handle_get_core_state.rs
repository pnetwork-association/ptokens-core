use common_sentinel::{ConfigT, SentinelConfig, SentinelError, WebSocketMessages, WebSocketMessagesEncodable};
use tokio::time::{sleep, Duration};

use crate::rpc_server::{
    constants::{WebSocketTx, STRONGBOX_TIMEOUT_MS},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_get_core_state(
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let mcids = vec![config.native().metadata_chain_id(), config.host().metadata_chain_id()];
        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetCoreState(mcids));
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "getting enclave state";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
