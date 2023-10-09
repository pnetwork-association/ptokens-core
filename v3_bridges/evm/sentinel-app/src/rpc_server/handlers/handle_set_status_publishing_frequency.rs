use std::str::FromStr;

use common_metadata::MetadataChainId;
use common_sentinel::{
    SentinelError,
    SentinelStatusError,
    StatusPublisherMessages,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    MAX_STATUS_PUBLISHING_FREQENCY,
    MIN_STATUS_PUBLISHING_FREQENCY,
};
use serde_json::{json, Value as Json};
use tokio::time::{sleep, Duration};

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

        if !(MIN_STATUS_PUBLISHING_FREQENCY..=MAX_STATUS_PUBLISHING_FREQENCY).contains(&frequency) {
            Err(SentinelStatusError::InvalidPublishingFrequency(frequency).into())
        } else {
            let msg = StatusPublisherMessages::SetStatusPublishingFreqency(frequency);
            status_tx.send(msg).await?;
            Ok(json!({"statusPublishingUpdateFrequency": frequency}))
        }
    }
}
