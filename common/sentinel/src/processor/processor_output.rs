use std::fmt;

use common_network_ids::NetworkId;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{get_utc_timestamp, SentinelError, SignedEvents};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Getters)]
pub struct ProcessorOutput {
    timestamp: u64,
    network_id: NetworkId,
    latest_block_num: u64,
    signed_events: SignedEvents,
}

impl ProcessorOutput {
    pub fn new(
        network_id: NetworkId,
        latest_block_num: u64,
        signed_events: SignedEvents,
    ) -> Result<Self, SentinelError> {
        Ok(Self {
            network_id,
            signed_events,
            latest_block_num,
            timestamp: get_utc_timestamp()?,
        })
    }

    pub fn has_user_ops(&self) -> bool {
        !self.signed_events.is_empty()
    }
}

impl TryFrom<Json> for ProcessorOutput {
    type Error = SentinelError;

    fn try_from(j: Json) -> Result<Self, SentinelError> {
        Ok(serde_json::from_value(j)?)
    }
}

impl fmt::Display for ProcessorOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error pretty printing `ProcessorOutput` json: {e}"),
        }
    }
}
