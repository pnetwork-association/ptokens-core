use common_sentinel::{
    BroadcastChannelMessages,
    NetworkId,
    NetworkIdError,
    SentinelError,
    SyncerBroadcastChannelMessages,
};
use serde_json::{json, Value as Json};

use crate::{
    rpc_server::{RpcCalls, RpcParams},
    type_aliases::BroadcastChannelTx,
};

impl RpcCalls {
    pub(crate) async fn handle_syncer_start_stop(
        broadcast_channel_tx: BroadcastChannelTx,
        params: RpcParams,
        stop: bool,
        core_cxn: bool,
    ) -> Result<Json, SentinelError> {
        let m = if stop { "stop" } else { "start" };
        debug!("handling {m} syncer rpc call...");
        Self::check_core_is_connected(core_cxn)?;

        if params.is_empty() {
            return Err(SentinelError::Custom("please provide 1 or more chain ids".to_string()));
        }

        let network_ids = params
            .iter()
            .map(NetworkId::try_from)
            .collect::<Result<Vec<NetworkId>, NetworkIdError>>()?;

        let jsons = network_ids
            .iter()
            .map(|network_id| json!({"status": format!("{m} message sent to {network_id} syncer")}))
            .collect::<Vec<Json>>();

        let json = json!(vec![jsons]);
        let msgs = network_ids
            .into_iter()
            .map(|network_id| {
                BroadcastChannelMessages::Syncer(
                    network_id,
                    if stop {
                        SyncerBroadcastChannelMessages::Stop
                    } else {
                        SyncerBroadcastChannelMessages::Start
                    },
                )
            })
            .collect::<Vec<BroadcastChannelMessages>>();

        for msg in msgs {
            broadcast_channel_tx.send(msg)?;
        }

        Ok(json)
    }
}
