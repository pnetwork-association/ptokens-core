use std::str::FromStr;

use common_metadata::MetadataChainId;
use common_sentinel::{SentinelError, WebSocketMessages, WebSocketMessagesEncodable, WebSocketMessagesError};
use tokio::time::{sleep, Duration};

use crate::rpc_server::{
    constants::{RpcParams, WebSocketTx, STRONGBOX_TIMEOUT_MS},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_get_status(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        Self::check_core_is_connected(core_cxn)?;

        let n = 1;
        let l = params.len();
        if l < n {
            return Err(WebSocketMessagesError::NeedMoreArgs { num_args: n }.into());
        }

        let mcids = params
            .iter()
            .map(|s| MetadataChainId::from_str(s).map_err(|_| WebSocketMessagesError::ParseMetadataChainId(s.into())))
            .collect::<Result<Vec<MetadataChainId>, WebSocketMessagesError>>()?;

        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetStatus(mcids));
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "getting status";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
