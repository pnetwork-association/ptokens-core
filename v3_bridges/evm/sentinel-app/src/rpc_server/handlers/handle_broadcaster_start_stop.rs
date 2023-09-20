use common_sentinel::{BroadcastChannelMessages, BroadcasterBroadcastChannelMessages, SentinelError};
use serde_json::{json, Value as Json};

use crate::rpc_server::{constants::BroadcastChannelTx, RpcCall};

impl RpcCall {
    pub(crate) async fn handle_broadcaster_start_stop(
        broadcast_channel_tx: BroadcastChannelTx,
        core_cxn: bool,
        start: bool,
    ) -> Result<Json, SentinelError> {
        debug!("handling stop syncer rpc call...");
        Self::check_core_is_connected(core_cxn)?;
        let json = json!({"status": format!("{} message sent to broadcaster", if start { "start" } else { "stop" })});
        let m = if start {
            BroadcasterBroadcastChannelMessages::Start
        } else {
            BroadcasterBroadcastChannelMessages::Stop
        };
        let msg = BroadcastChannelMessages::Broadcaster(m);
        broadcast_channel_tx.send(msg)?;
        Ok(json)
    }
}
