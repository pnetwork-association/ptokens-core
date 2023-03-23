use common_eth::{EthLog, EthReceipts};
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

use crate::{AddressAndTopic, AddressesAndTopics};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Deref, Constructor)]
pub struct RelevantLogs(Vec<EthLog>);

impl RelevantLogs {
    pub fn from_eth_receipts(addresses_and_topics: &AddressesAndTopics, receipts: &EthReceipts) -> Self {
        let mut logs: Vec<EthLog> = vec![];
        for receipt in receipts.iter() {
            for log in receipt.logs.iter() {
                for AddressAndTopic { address, topic } in addresses_and_topics.iter() {
                    if !log.topics.is_empty() && &log.address == address && &log.topics[0] == topic {
                        logs.push(log.clone())
                    }
                }
            }
        }
        Self::new(logs)
    }
}
