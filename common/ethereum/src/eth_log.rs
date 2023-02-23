#[cfg(test)]
use std::str::FromStr;

use common::types::{Bytes, Result};
#[cfg(test)]
use common::AppError;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, Bloom, BloomInput, H256 as EthHash};
use rlp::{Encodable, RlpStream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::{
    eth_receipt::EthReceiptJson,
    eth_utils::{convert_hex_strings_to_h256s, convert_hex_to_bytes, convert_hex_to_eth_address},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthLogJson {
    pub data: String,
    pub address: String,
    pub topics: Vec<String>,
}

#[cfg(test)]
impl FromStr for EthLogJson {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str::<EthLogJson>(s)?)
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize, Constructor)]
pub struct EthLog {
    pub address: EthAddress,
    pub topics: Vec<EthHash>,
    pub data: Bytes,
}

impl EthLogExt for EthLog {
    fn get_address(&self) -> EthAddress {
        self.address
    }

    fn get_topics(&self) -> Vec<EthHash> {
        self.topics.clone()
    }

    fn get_data(&self) -> Bytes {
        self.data.clone()
    }
}

#[cfg(test)]
impl FromStr for EthLog {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        EthLogJson::from_str(s).and_then(|ref json| Self::from_json(json))
    }
}

impl EthLog {
    pub fn from_json(log_json: &EthLogJson) -> Result<Self> {
        Ok(EthLog {
            data: convert_hex_to_bytes(&log_json.data)?,
            address: convert_hex_to_eth_address(&log_json.address)?,
            topics: convert_hex_strings_to_h256s(&log_json.topics)?,
        })
    }

    pub fn to_json(&self) -> Result<JsonValue> {
        let topic_strings = self
            .topics
            .iter()
            .map(|topic_hash| format!("0x{}", hex::encode(topic_hash.as_bytes())))
            .collect::<Vec<String>>();
        Ok(json!({
            "topics": topic_strings,
            "data": format!("0x{}", hex::encode(self.data.clone())),
            "address": format!("0x{}", hex::encode(self.address.as_bytes())),
        }))
    }

    pub fn get_bloom(&self) -> Bloom {
        self.topics.iter().fold(
            Bloom::from(BloomInput::Raw(self.address.as_bytes())),
            |mut bloom, topic| {
                bloom.accrue(BloomInput::Raw(topic.as_bytes()));
                bloom
            },
        )
    }

    pub fn is_from_address(&self, address: &EthAddress) -> bool {
        self.address == *address
    }

    pub fn is_from_address_and_contains_topic(&self, address: &EthAddress, topic: &EthHash) -> bool {
        self.is_from_address(address) && self.contains_topic(topic)
    }

    pub fn is_from_one_of_addresses(&self, addresses: Vec<EthAddress>) -> bool {
        addresses.iter().any(|a| self.is_from_address(a))
    }
}

impl Encodable for EthLog {
    fn rlp_append(&self, rlp_stream: &mut RlpStream) {
        rlp_stream
            .begin_list(3)
            .append(&self.address)
            .append_list(&self.topics)
            .append(&self.data);
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct EthLogs(pub Vec<EthLog>);

impl EthLogs {
    pub fn get_bloom(&self) -> Bloom {
        self.0.iter().fold(Bloom::default(), |mut bloom, log| {
            bloom.accrue_bloom(&log.get_bloom());
            bloom
        })
    }

    pub fn from_receipt_json(json: &EthReceiptJson) -> Result<Self> {
        Ok(Self(
            json.logs
                .iter()
                .map(EthLog::from_json)
                .collect::<Result<Vec<EthLog>>>()?,
        ))
    }

    pub fn contain_topic(&self, topic: &EthHash) -> bool {
        self.0.iter().any(|log| log.contains_topic(topic))
    }

    pub fn contain_address(&self, address: &EthAddress) -> bool {
        self.0.iter().any(|log| log.is_from_address(address))
    }

    pub fn filter_for_those_from_address_containing_topic(&self, address: &EthAddress, topic: &EthHash) -> Self {
        EthLogs::new(
            self.iter()
                .cloned()
                .filter(|log| log.is_from_address_and_contains_topic(address, topic))
                .collect(),
        )
    }
}

pub trait EthLogExt {
    fn get_data(&self) -> Bytes;
    fn get_address(&self) -> EthAddress;
    fn get_topics(&self) -> Vec<EthHash>;

    fn get_event_signature(&self) -> Result<EthHash> {
        self.check_has_x_topics(1).map(|_| self.get_topics()[0])
    }

    fn contains_topic(&self, topic: &EthHash) -> bool {
        self.get_topics().iter().any(|log_topic| log_topic == topic)
    }

    fn check_has_x_topics(&self, x: usize) -> Result<()> {
        if self.get_topics().len() >= x {
            Ok(())
        } else {
            Err(format!("Log does not have {} topics!", x).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        get_expected_log,
        get_sample_contract_address,
        get_sample_contract_topic,
        get_sample_eth_submission_material_json,
        get_sample_log_with_desired_address,
        get_sample_log_with_desired_topic,
        get_sample_log_without_desired_address,
        get_sample_logs_with_desired_topic,
        get_sample_logs_without_desired_topic,
        get_sample_receipt_with_desired_address,
        get_sample_receipt_without_desired_address,
        SAMPLE_RECEIPT_INDEX,
    };

    #[test]
    fn should_get_logs_bloom_from_logs() {
        let expected_bloom = "00000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000010000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000800000000000000000000010000000000000000008000000000000000000000000000000000000000000000200000003000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000020000000";
        let expected_bloom_bytes = &hex::decode(expected_bloom).unwrap()[..];
        let eth_block_and_receipt_json = get_sample_eth_submission_material_json().unwrap();
        let receipt_json = eth_block_and_receipt_json.receipts[SAMPLE_RECEIPT_INDEX].clone();
        let logs = EthLogs::from_receipt_json(&receipt_json).unwrap();
        let result = logs.get_bloom();
        assert_eq!(result.as_bytes(), expected_bloom_bytes)
    }

    #[test]
    fn should_get_logs_from_receipt_json() {
        let expected_result = get_expected_log();
        let eth_block_and_receipt_json = get_sample_eth_submission_material_json().unwrap();
        let result = EthLogs::from_receipt_json(&eth_block_and_receipt_json.receipts[SAMPLE_RECEIPT_INDEX]).unwrap();
        assert_eq!(result.0[0], expected_result);
    }

    #[test]
    fn should_get_log_from_log_json_correctly() {
        let eth_block_and_receipt_json = get_sample_eth_submission_material_json().unwrap();
        let log_json = eth_block_and_receipt_json.receipts[SAMPLE_RECEIPT_INDEX].logs[0].clone();
        let result = EthLog::from_json(&log_json).unwrap();
        let expected_result = get_expected_log();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_encode_eth_log_as_json() {
        let log = get_sample_log_with_desired_topic();
        let result = log.to_json().unwrap();
        let expected_result = json!({
            "address": "0x60a640e2d10e020fee94217707bfa9543c8b59e0",
            "data": "0x00000000000000000000000000000000000000000000000589ba7ab174d54000",
            "topics": vec![
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                "0x000000000000000000000000250abfa8bc8371709fa4b601d821b1421667a886",
                "0x0000000000000000000000005a7dd68907e103c3239411dae0b0eef968468ef2",
            ]
        });
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_true_if_log_contains_desired_topic() {
        let log = get_sample_log_with_desired_topic();
        let topic = get_sample_contract_topic();
        let result = log.contains_topic(&topic);
        assert!(result);
    }

    #[test]
    fn sample_logs_with_desired_topic_should_contain_topic() {
        let logs = get_sample_logs_with_desired_topic();
        let topic = get_sample_contract_topic();
        let result = logs.contain_topic(&topic);
        assert!(result);
    }

    #[test]
    fn sample_logs_without_desired_topic_should_contain_topic() {
        let logs = get_sample_logs_without_desired_topic();
        let topic = get_sample_contract_topic();
        let result = logs.contain_topic(&topic);
        assert!(!result);
    }

    #[test]
    fn sample_log_receipt_with_desired_address_should_return_true() {
        let log = get_sample_log_with_desired_address();
        let address = get_sample_contract_address();
        let result = log.is_from_address(&address);
        assert!(result);
    }

    #[test]
    fn sample_log_without_desired_address_should_return_false() {
        let log = get_sample_log_without_desired_address();
        let address = get_sample_contract_address();
        let result = log.is_from_address(&address);
        assert!(!result);
    }

    #[test]
    fn sample_receipt_with_desired_address_should_return_true() {
        let receipt = get_sample_receipt_with_desired_address();
        let address = get_sample_contract_address();
        let result = receipt.logs.contain_address(&address);
        assert!(result);
    }

    #[test]
    fn sample_receipt_without_desired_address_should_return_false() {
        let receipt = get_sample_receipt_without_desired_address();
        let address = get_sample_contract_address();
        let result = receipt.logs.contain_address(&address);
        assert!(!result);
    }

    #[test]
    fn log_should_contain_desired_address_and_topic() {
        let desired_topic = EthHash::from_slice(
            &hex::decode("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef").unwrap(),
        );
        let log = get_sample_log_with_desired_address();
        let desired_address = get_sample_contract_address();
        let result = log.is_from_address_and_contains_topic(&desired_address, &desired_topic);
        assert!(result);
    }

    #[test]
    fn should_filter_logs_for_those_from_desired_address_containing_topic() {
        let desired_topic = EthHash::from_slice(
            &hex::decode("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef").unwrap(),
        );
        let desired_address = get_sample_contract_address();
        let expected_log = get_sample_log_with_desired_address();
        let expected_result = EthLogs::new(vec![expected_log.clone()]);
        let logs = EthLogs::new(vec![expected_log, get_sample_log_without_desired_address()]);
        let num_logs_before = logs.len();
        assert_eq!(num_logs_before, 2);
        let result = logs.filter_for_those_from_address_containing_topic(&desired_address, &desired_topic);
        let num_logs_after = result.len();
        assert_eq!(num_logs_after, 1);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_eth_log_from_str() {
        let s = "{\"address\":\"0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9\",\"topics\":[\"0xc03be660a5421fb17c93895da9db564bd4485d475f0d8b3175f7d55ed421bebb\"],\"data\":\"0x0000000000000000000000005fc8d32690cc91d4c39d9d3abcbd16989f87570700000000000000000000000070997970c51812dc3a010c7d01b50e0d17dc79c8000000000000000000000000000000000000000000000000000000000000053900000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001200069c3220000000000000000000000000000000000000000000000000000000000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c616e656f736164647265737300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\"}";
        let result = EthLog::from_str(s);
        assert!(result.is_ok());
    }
}
