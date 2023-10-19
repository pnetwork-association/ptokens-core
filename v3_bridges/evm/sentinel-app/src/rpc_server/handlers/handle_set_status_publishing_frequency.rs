use common_sentinel::{sanity_check_frequency, SentinelError, StatusPublisherMessages};
use serde_json::{json, Value as Json};

use crate::{
    rpc_server::{RpcCall, RpcParams},
    type_aliases::StatusPublisherTx,
};

impl RpcCall {
    pub(crate) async fn handle_set_status_publishing_frequency(
        params: RpcParams,
        status_tx: StatusPublisherTx,
    ) -> Result<Json, SentinelError> {
        debug!("handling set status publishing frequency rpc call...");
        let checked_params = Self::check_params(params, 1)?;
        let frequency = checked_params[0].parse::<u64>()?;
        let sanity_checked_frequency = sanity_check_frequency(frequency)?;
        let msg = StatusPublisherMessages::SetStatusPublishingFreqency(sanity_checked_frequency);
        status_tx.send(msg).await?;
        Ok(json!({"statusPublishingUpdateFrequency": sanity_checked_frequency}))
    }
}
