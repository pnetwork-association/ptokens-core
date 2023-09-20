use common_sentinel::{SentinelError, WebSocketMessages, WebSocketMessagesEncodable};
use tokio::time::{sleep, Duration};

use crate::rpc_server::{
    constants::{RpcParams, WebSocketTx, STRONGBOX_TIMEOUT_MS},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_delete(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling db delete...");
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;
        let encodable_msg = WebSocketMessagesEncodable::try_from(Self::create_args("delete", checked_params))?;
        let (msg, rx) = WebSocketMessages::new(encodable_msg);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "deleting value from db";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }

    pub(crate) async fn handle_get(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling db get...");
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;
        let encodable_msg = WebSocketMessagesEncodable::try_from(Self::create_args("get", checked_params))?;
        let (msg, rx) = WebSocketMessages::new(encodable_msg);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "getting value from db";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }

    pub(crate) async fn handle_put(
        websocket_tx: WebSocketTx,
        params: RpcParams,
        core_cxn: bool,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        debug!("handling db put...");
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 2)?;
        let encodable_msg = WebSocketMessagesEncodable::try_from(Self::create_args("put", checked_params))?;
        let (msg, rx) = WebSocketMessages::new(encodable_msg);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "putting value in db";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }
}
