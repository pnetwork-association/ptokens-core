use common_sentinel::{BroadcastChannelMessages, SentinelError, UserOpCancellerBroadcastChannelMessages};
use serde_json::{json, Value as Json};

use crate::{rpc_server::RpcCalls, type_aliases::BroadcastChannelTx};

impl RpcCalls {
    pub(crate) async fn handle_user_op_canceller_start_stop(
        broadcast_channel_tx: BroadcastChannelTx,
        core_cxn: bool,
        start: bool,
    ) -> Result<Json, SentinelError> {
        debug!("handling user op canceller start/stop...");
        Self::check_core_is_connected(core_cxn)?;
        let json =
            json!({"status": format!("{} message sent to user op canceller", if start { "start" } else { "stop" })});
        let m = if start {
            UserOpCancellerBroadcastChannelMessages::Start
        } else {
            UserOpCancellerBroadcastChannelMessages::Stop
        };
        let msg = BroadcastChannelMessages::UserOpCanceller(m);
        broadcast_channel_tx.send(msg)?;
        Ok(json)
    }
}
