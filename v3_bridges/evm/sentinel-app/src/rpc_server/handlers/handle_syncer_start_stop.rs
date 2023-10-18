use std::str::FromStr;

use common::AppError;
use common_metadata::MetadataChainId;
use common_sentinel::{BroadcastChannelMessages, SentinelError, SyncerBroadcastChannelMessages};
use serde_json::{json, Value as Json};

use crate::{
    rpc_server::{RpcCall, RpcParams},
    type_aliases::BroadcastChannelTx,
};

impl RpcCall {
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

        let mcids = params
            .iter()
            .map(|s| MetadataChainId::from_str(s))
            .collect::<Result<Vec<MetadataChainId>, AppError>>()?;
        let jsons = mcids
            .iter()
            .map(|mcid| json!({"status": format!("{m} message sent to {mcid} syncer")}))
            .collect::<Vec<Json>>();
        let json = json!(vec![jsons]);
        let msgs = mcids
            .into_iter()
            .map(|mcid| {
                BroadcastChannelMessages::Syncer(
                    mcid,
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
