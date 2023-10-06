use common_sentinel::{
    BroadcastChannelMessages,
    SentinelError,
    StatusPublisherBroadcastChannelMessages,
    StatusPublisherMessages,
};
use serde_json::{json, Value as Json};

use crate::{rpc_server::RpcCall, type_aliases::BroadcastChannelTx};

impl RpcCall {
    pub(crate) async fn handle_status_publisher_start_stop(
        broadcast_channel_tx: BroadcastChannelTx,
        start: bool,
    ) -> Result<Json, SentinelError> {
        debug!("handling status publisher start/stop...");
        let json = json!({"status": format!("{} message sent to status publisher via broadcast channel", if start { "start" } else { "stop" })});

        let m = if start {
            StatusPublisherBroadcastChannelMessages::Start
        } else {
            StatusPublisherBroadcastChannelMessages::Stop
        };

        let msg = BroadcastChannelMessages::Status(m);

        // NOTE: We use the broadcast channel rather than the specific status publisher channel because the
        // loop handling broadcaster channel messages runs regardless of whether the module is turned on or off.
        broadcast_channel_tx.send(msg)?;
        Ok(json)
    }
}
