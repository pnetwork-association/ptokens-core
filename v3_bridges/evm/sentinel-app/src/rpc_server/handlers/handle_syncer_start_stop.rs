use std::str::FromStr;

use common_metadata::MetadataChainId;
use common_sentinel::{BroadcastChannelMessages, SentinelError, SyncerBroadcastChannelMessages};
use serde_json::{json, Value as Json};

use crate::rpc_server::{
    constants::{BroadcastChannelTx, RpcParams},
    RpcCall,
};

impl RpcCall {
    pub(crate) async fn handle_syncer_start_stop(
        broadcast_channel_tx: BroadcastChannelTx,
        params: RpcParams,
        stop: bool,
        core_cxn: bool,
    ) -> Result<Json, SentinelError> {
        debug!("handling stop syncer rpc call...");
        Self::check_core_is_connected(core_cxn)?;
        let checked_params = Self::check_params(params, 1)?;
        let mcid = MetadataChainId::from_str(&checked_params[0])?;
        let syncer_msg = if stop {
            SyncerBroadcastChannelMessages::Stop
        } else {
            SyncerBroadcastChannelMessages::Start
        };
        let m = if stop { "stop" } else { "start" };
        let json = json!({"status": format!("{m} message sent to {mcid} syncer")});
        let broadcast_channel_msg = BroadcastChannelMessages::Syncer(mcid, syncer_msg);
        broadcast_channel_tx.send(broadcast_channel_msg)?;
        Ok(json)
    }
}
