use std::{fmt, iter::IntoIterator, str::FromStr, time::Duration};

use common::{Byte, Bytes};
use common_eth::{EthLog, EthLogs, EthReceipts};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::{AddressAndTopic, SentinelError};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct RelevantLogs(Vec<RelevantLogsFromBlock>);

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct RelevantLogsFromBlock {
    block_num: u64,
    timestamp: Duration,
    logs: EthLogs,
}

#[cfg(test)]
impl RelevantLogsFromBlock {
    pub fn set_timestamp(&mut self, ts: Duration) {
        self.timestamp = ts;
    }
}

impl RelevantLogsFromBlock {
    pub fn from_eth_receipts<A>(
        block_num: u64,
        timestamp: Duration,
        receipts: &EthReceipts,
        addresses_and_topics: &A,
    ) -> Self
    where
        for<'a> &'a A: IntoIterator<Item = &'a AddressAndTopic>,
    {
        let mut logs: Vec<EthLog> = vec![];
        for receipt in receipts.iter() {
            for log in receipt.logs.iter() {
                for AddressAndTopic { address, topic } in addresses_and_topics.into_iter() {
                    if !log.topics.is_empty() && &log.address == address && &log.topics[0] == topic {
                        logs.push(log.clone());
                    }
                }
            }
        }
        Self::new(block_num, timestamp, EthLogs::new(logs))
    }
}

macro_rules! make_log_structs {
    ($($prefix:ident),* $(,)?) => {
        paste! {
            $(
                #[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor)]
                pub struct [< $prefix:camel RelevantLogs >](RelevantLogs);

                impl fmt::Display for [< $prefix:camel RelevantLogs >] {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        match serde_json::to_string_pretty(self) {
                            Ok(s) => write!(f, "{s}"),
                            Err(e) => write!(f, "Error convert `HostRelevantLogs` to string: {e}",),
                        }
                    }
                }


                impl FromStr for [< $prefix:camel RelevantLogs >]{
                    type Err = SentinelError;

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        Ok(serde_json::from_str(s)?)
                    }
                }

                impl TryInto<Bytes> for [< $prefix:camel RelevantLogs >] {
                    type Error = SentinelError;

                    fn try_into(self) -> Result<Bytes, Self::Error> {
                        Ok(serde_json::to_vec(&self)?)
                    }
                }

                impl TryFrom<&[Byte]> for [< $prefix:camel RelevantLogs >] {
                    type Error = SentinelError;

                    fn try_from(b: &[Byte]) -> Result<Self, Self::Error> {
                        Ok(serde_json::from_slice(b)?)
                    }
                }

                impl TryFrom<Bytes> for [< $prefix:camel RelevantLogs >] {
                    type Error = SentinelError;

                    fn try_from(b: Bytes) -> Result<Self, Self::Error> {
                        Ok(serde_json::from_slice(&b)?)
                    }
                }
            )*
        }
    }
}

make_log_structs!(HOST, NATIVE);
