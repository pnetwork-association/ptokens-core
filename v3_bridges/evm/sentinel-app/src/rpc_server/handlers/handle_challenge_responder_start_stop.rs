use common_sentinel::{BroadcastChannelMessages, ChallengeResponderBroadcastChannelMessages, SentinelError};
use serde_json::{json, Value as Json};

use crate::{rpc_server::RpcCall, type_aliases::BroadcastChannelTx};

impl RpcCall {
    pub(crate) async fn handle_challenge_responder_start_stop(
        broadcast_channel_tx: BroadcastChannelTx,
        start: bool,
    ) -> Result<Json, SentinelError> {
        debug!("handling challenge responder start/stop...");
        let json = json!({"status": format!("{} message sent to challenge responder via broadcast channel", if start { "start" } else { "stop" })});

        let m = if start {
            ChallengeResponderBroadcastChannelMessages::Start
        } else {
            ChallengeResponderBroadcastChannelMessages::Stop
        };

        let msg = BroadcastChannelMessages::ChallengeResponder(m);

        // NOTE: We use the broadcast channel rather than the specific component's channel because the
        // loop handling the broadcaster channel messages runs regardless of whether the module is turned on or off.
        broadcast_channel_tx.send(msg)?;
        Ok(json)
    }
}
