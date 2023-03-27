use std::{fmt, str::FromStr, time::Duration};

use common::{Byte, Bytes};
use common_eth::{EthLog, EthLogs, EthReceipts};
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{SentinelError, USER_OPERATION_TOPIC};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor, Deref)]
pub struct RelevantLogs(Vec<RelevantLogsFromBlock>);

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct RelevantLogsFromBlock {
    block_num: u64,
    timestamp: Duration,
    logs: EthLogs, // FIXME change this to UserOperations (by converting the log to UserOperation)
}

#[cfg(test)]
impl RelevantLogsFromBlock {
    pub fn set_timestamp(&mut self, ts: Duration) {
        self.timestamp = ts;
    }
}

impl RelevantLogsFromBlock {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_eth_receipts(
        block_num: u64,
        timestamp: Duration,
        receipts: &EthReceipts,
        state_manager: &EthAddress,
    ) -> Self {
        let mut logs: Vec<EthLog> = vec![];
        for receipt in receipts.iter() {
            for log in receipt.logs.iter() {
                if !log.topics.is_empty() && &log.address == state_manager && log.topics[0] == *USER_OPERATION_TOPIC {
                    logs.push(log.clone());
                }
            }
        }
        Self::new(block_num, timestamp, EthLogs::new(logs))
    }
}

impl RelevantLogs {
    pub fn add(&mut self, other: Self) {
        let a = self.0.clone();
        let b = other.0;
        self.0 = [a, b].concat();
    }
}

impl fmt::Display for RelevantLogs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "Error convert `HostRelevantLogs` to string: {e}",),
        }
    }
}

impl FromStr for RelevantLogs {
    type Err = SentinelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

impl TryInto<Bytes> for RelevantLogs {
    type Error = SentinelError;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        Ok(serde_json::to_vec(&self)?)
    }
}

impl TryFrom<&[Byte]> for RelevantLogs {
    type Error = SentinelError;

    fn try_from(b: &[Byte]) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(b)?)
    }
}

impl TryFrom<Bytes> for RelevantLogs {
    type Error = SentinelError;

    fn try_from(b: Bytes) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(&b)?)
    }
}
