use std::{result::Result, time::SystemTime};

use common_eth::{EthSubmissionMaterial, EthSubmissionMaterials};
use derive_getters::Getters;
use ethereum_types::{Address as EthAddress, U256};
use jsonrpsee::ws_client::WsClient;
use serde_json::Value as Json;
use thiserror::Error;

use crate::{endpoints::Endpoints, Bpm, NetworkId, ProcessorOutput, SentinelConfig, SentinelError};

#[derive(Debug, Clone, Getters)]
pub struct Batch {
    bpm: Bpm,
    confs: u64,
    block_num: u64,
    batch_size: u64,
    sleep_duration: u64,
    batch_duration: u64,
    endpoints: Endpoints,
    network_id: NetworkId,
    pnetwork_hub: EthAddress,
    pre_filter_receipts: bool,
    batching_is_disabled: bool,
    single_submissions_flag: bool,
    batch: EthSubmissionMaterials,
    last_submitted_timestamp: SystemTime,
    governance_address: Option<EthAddress>,
    receipt_filtering_addresses: Vec<EthAddress>,
}

impl Default for Batch {
    fn default() -> Self {
        Self {
            confs: 1,
            block_num: 0,
            batch_size: 1,
            sleep_duration: 0,
            bpm: Bpm::default(),
            batch_duration: 5 * 60, // NOTE: 5mins
            governance_address: None,
            pre_filter_receipts: false,
            batching_is_disabled: false,
            single_submissions_flag: false,
            endpoints: Endpoints::default(),
            network_id: NetworkId::default(),
            pnetwork_hub: EthAddress::default(),
            receipt_filtering_addresses: vec![],
            batch: EthSubmissionMaterials::default(),
            last_submitted_timestamp: SystemTime::now(),
        }
    }
}

impl Batch {
    pub fn set_single_submissions_flag(&mut self) {
        self.single_submissions_flag = true;
    }

    pub fn set_confs(&mut self, n: u64) {
        self.confs = n;
    }

    pub fn get_confs(&self) -> u64 {
        self.confs
    }

    pub fn increment_block_num(&mut self) {
        self.block_num += 1;
    }

    pub fn set_block_num(&mut self, n: u64) {
        self.block_num = n;
    }

    pub fn get_block_num(&self) -> u64 {
        self.block_num
    }

    pub fn update_bpm_from_json(&mut self, j: Json) {
        let err_msg = format!("error converting json: '{j}' to processor output:");
        match ProcessorOutput::try_from(j) {
            Ok(ref o) => self.update_bpm(o),
            Err(e) => {
                warn!("{err_msg}: {e}");
                warn!("not updating syncer bpm for network {}", self.bpm.network_id());
            },
        }
    }

    pub fn update_bpm(&mut self, o: &ProcessorOutput) {
        self.bpm.push(o)
    }

    pub fn new(nid: NetworkId) -> Self {
        Self {
            bpm: Bpm::new(nid),
            ..Default::default()
        }
    }

    pub async fn check_endpoint(&self) -> Result<(), SentinelError> {
        self.endpoints.check_endpoint(30).await
    }

    pub async fn get_first_ws_client(&self) -> Result<WsClient, SentinelError> {
        self.endpoints.get_first_ws_client().await
    }

    pub fn get_sleep_duration(&self) -> u64 {
        self.sleep_duration
    }

    pub fn new_from_config(network_id: NetworkId, config: &SentinelConfig) -> Result<Self, SentinelError> {
        info!("getting batch from config...");

        let pre_filter_receipts = config.pre_filter_receipts(&network_id)?;

        let pnetwork_hub = config.pnetwork_hub_from_network_id(&network_id)?;

        let governance_address = config.governance_address(&network_id);

        let receipt_filtering_addresses = if let Some(ref address) = governance_address {
            vec![*address, pnetwork_hub]
        } else {
            vec![pnetwork_hub]
        };

        let sleep_duration = config.sleep_duration(&network_id)?;

        let endpoints = config.endpoints(&network_id)?;

        let res = Self {
            endpoints,
            network_id,
            pnetwork_hub,
            sleep_duration,
            governance_address,
            pre_filter_receipts,
            bpm: Bpm::new(network_id),
            receipt_filtering_addresses,
            batch_size: config.batch_size(&network_id)?,
            batch_duration: config.batch_duration(&network_id)?,
            ..Default::default()
        };

        if res.endpoints.is_empty() {
            Err(SentinelError::Batching(BatchingError::NoEndpoint(network_id)))
        } else {
            Ok(res)
        }
    }

    pub fn enable_batching(&mut self) {
        self.batching_is_disabled = false;
    }

    pub fn disable_batching(&mut self) {
        self.batching_is_disabled = true;
    }

    pub fn batching_is_enabled(&self) -> bool {
        !self.batching_is_disabled
    }

    pub fn set_time_of_last_submission(&mut self) {
        self.last_submitted_timestamp = SystemTime::now();
    }

    pub fn get_time_of_last_submission(&self) -> SystemTime {
        self.last_submitted_timestamp
    }

    pub fn push(&mut self, sub_mat: EthSubmissionMaterial) {
        if self.pre_filter_receipts {
            self.batch
                .push(sub_mat.remove_receipts_if_no_logs_from_addresses(&self.receipt_filtering_addresses));
        } else {
            self.batch.push(sub_mat)
        }
    }

    pub fn get_pnetwork_hub(&self) -> &EthAddress {
        &self.pnetwork_hub
    }

    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    pub fn drain(&mut self) {
        if self.single_submissions_flag {
            // If we're draining a batch it's due to a successful submission, so we reset this flag.
            self.single_submissions_flag = false;
        };
        self.batch = EthSubmissionMaterials::new(vec![]);
        self.set_time_of_last_submission()
    }

    pub fn size_in_blocks(&self) -> u64 {
        self.batch.len() as u64
    }

    pub fn get_seconds_since_last_submission(&self) -> u64 {
        match self.last_submitted_timestamp.elapsed() {
            Ok(d) => d.as_secs(),
            Err(e) => {
                // NOTE: We default to u64::MAX here because that will mean the batch is always ready
                // to submit in case of this error, which is preferable to the batch never being ready!
                warn!("error getting time since last submission: {e}");
                u64::MAX
            },
        }
    }

    pub fn is_ready_to_submit(&self) -> bool {
        if self.is_empty() {
            info!("{} batch not ready to submit because it's empty", self.network_id());
            return false;
        } else if self.single_submissions_flag {
            info!(
                "{} batch set to single submission so it's ready to submit",
                self.network_id()
            );
            return true;
        }

        let size = self.size_in_blocks();
        let size_limit = self.batch_size;
        if size >= size_limit {
            info!(
                "{} batch has sufficient blocks to submit! (blocks: {size}, limit: {size_limit})",
                self.network_id()
            );
            return true;
        }

        let time_limit = self.batch_duration;
        let time = self.get_seconds_since_last_submission();
        if time >= time_limit {
            info!(
                "{} batch ready to submit because enough time has elapsed",
                self.network_id()
            );
            return true;
        }

        let pct_full = (size as f64 / size_limit as f64) * 100_f64;
        let pct_time = (time as f64 / time_limit as f64) * 100_f64;
        info!(
            "{} batch not ready to submit yet! ({size} blocks, {pct_full:.2}% full, {pct_time:.2}% time)",
            self.network_id()
        );
        false
    }

    pub fn check_is_chained(self) -> Result<Self, SentinelError> {
        let num_blocks_in_batch = self.size_in_blocks() as usize;
        if num_blocks_in_batch < 2 {
            info!(
                "no need to check {} batch chaining - it contains too few blocks to matter!",
                self.network_id()
            );
            Ok(self)
        } else {
            info!("checking {} batch is chained correctly...", self.network_id());
            let mut i = num_blocks_in_batch - 1;
            while i > 0 {
                if self.batch[i].get_parent_hash()? != self.batch[i - 1].get_block_hash()? {
                    let n_1 = self.batch[i].get_block_number()?;
                    let n_2 = self.batch[i - 1].get_block_number()?;
                    return Err(SentinelError::Batching(BatchingError::UnchainedBlocks {
                        block_num: n_1,
                        parent_block_num: n_2,
                    }));
                }
                i -= 1;
            }
            info!("{} batch is chained correctly", self.network_id());
            Ok(self)
        }
    }

    pub fn to_submission_material(&self) -> EthSubmissionMaterials {
        self.batch.clone()
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum BatchingError {
    #[error("blocks {parent_block_num} and {block_num} are not correctly chained")]
    UnchainedBlocks { block_num: U256, parent_block_num: U256 },

    #[error("no endpoints for network {0}")]
    NoEndpoint(NetworkId),
}

#[cfg(test)]
mod tests {
    use common_eth::{convert_hex_to_eth_address, EthLog, EthLogs, EthReceipt, EthReceipts};

    use super::*;
    use crate::test_utils::get_sample_batch;

    #[test]
    fn should_enable_batching() {
        let nid = NetworkId::try_from("eth").unwrap();
        let mut batch = Batch::new(nid);
        batch.disable_batching();
        assert!(!batch.batching_is_enabled());
        batch.enable_batching();
        let result = batch.batching_is_enabled();
        assert!(result);
    }

    #[test]
    fn should_disable_batching() {
        let nid = NetworkId::try_from("eth").unwrap();
        let mut batch = Batch::new(nid);
        batch.disable_batching();
        let result = batch.batching_is_enabled();
        assert!(!result);
    }

    #[test]
    fn should_set_time_of_last_submission() {
        let nid = NetworkId::try_from("eth").unwrap();
        let mut batch = Batch::new(nid);
        let timestamp_before = batch.get_time_of_last_submission();
        batch.set_time_of_last_submission();
        let result = batch.get_time_of_last_submission();
        assert!(result > timestamp_before);
    }

    #[test]
    fn should_push_to_batch() {
        let nid = NetworkId::try_from("eth").unwrap();
        let mut batch = Batch::new(nid);
        assert!(batch.is_empty());
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert!(!batch.is_empty());
    }

    #[test]
    fn should_drain_batch() {
        let nid = NetworkId::try_from("eth").unwrap();
        let mut batch = Batch::new(nid);
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert!(!batch.is_empty());
        batch.drain();
        assert!(batch.is_empty());
    }

    #[test]
    fn should_get_size_in_blocks_of_batch() {
        let nid = NetworkId::try_from("eth").unwrap();
        let mut batch = Batch::new(nid);
        assert_eq!(batch.size_in_blocks(), 0);
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert_eq!(batch.size_in_blocks(), 1);
    }

    #[test]
    fn pushed_block_should_have_receipts_if_they_contain_pertinent_logs() {
        let address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let log = EthLog {
            address,
            ..Default::default()
        };
        let logs = EthLogs::new(vec![log]);
        let receipt = EthReceipt {
            logs,
            ..Default::default()
        };
        let receipts = EthReceipts::new(vec![receipt]);
        let sub_mat = EthSubmissionMaterial {
            receipts: receipts.clone(),
            ..Default::default()
        };
        let mut batch = Batch {
            pre_filter_receipts: true,
            receipt_filtering_addresses: vec![address],
            ..Default::default()
        };
        batch.push(sub_mat);
        assert_eq!(batch.batch[0].receipts, receipts);
    }

    #[test]
    fn pushed_block_should_not_have_receipts_if_they_do_not_contain_pertinent_logs_and_pre_filtering_is_enabled() {
        let address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let other_address = convert_hex_to_eth_address("0x690b9a9e9aa1c9db991c7721a92d351db4fac990").unwrap();
        let log = EthLog {
            address,
            ..Default::default()
        };
        let logs = EthLogs::new(vec![log]);
        let receipt = EthReceipt {
            logs,
            ..Default::default()
        };
        let receipts = EthReceipts::new(vec![receipt]);
        let sub_mat = EthSubmissionMaterial {
            receipts,
            ..Default::default()
        };
        let mut batch = Batch {
            pre_filter_receipts: true,
            receipt_filtering_addresses: vec![other_address],
            ..Default::default()
        };
        batch.push(sub_mat);
        assert!(batch.batch[0].receipts.is_empty());
    }

    #[test]
    fn pushed_block_should_have_receipts_if_they_do_not_contain_pertinent_logs_but_pre_filtering_is_disabled() {
        let address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let other_address = convert_hex_to_eth_address("0x690b9a9e9aa1c9db991c7721a92d351db4fac990").unwrap();
        let log = EthLog {
            address,
            ..Default::default()
        };
        let logs = EthLogs::new(vec![log]);
        let receipt = EthReceipt {
            logs,
            ..Default::default()
        };
        let receipts = EthReceipts::new(vec![receipt]);
        let sub_mat = EthSubmissionMaterial {
            receipts: receipts.clone(),
            ..Default::default()
        };
        let mut batch = Batch {
            pre_filter_receipts: false,
            receipt_filtering_addresses: vec![other_address],
            ..Default::default()
        };
        batch.push(sub_mat);
        assert_eq!(batch.batch[0].receipts, receipts);
    }

    #[test]
    fn should_pass_is_chained_check_if_batch_is_empty() {
        let nid = NetworkId::try_from("eth").unwrap();
        let batch = Batch::new(nid);
        assert!(batch.check_is_chained().is_ok())
    }

    #[test]
    fn should_pass_is_chained_check_if_batch_has_one_member() {
        let nid = NetworkId::try_from("eth").unwrap();
        let mut batch = Batch::new(nid);
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert_eq!(batch.size_in_blocks(), 1);
        assert!(batch.check_is_chained().is_ok())
    }

    #[test]
    fn should_pass_is_chained_check_if_is_chained_correctly() {
        let batch = get_sample_batch();
        assert!(batch.check_is_chained().is_ok());
    }

    #[test]
    fn should_fail_is_chained_check_if_is_not_chained_correctly() {
        let mut batch = get_sample_batch();
        batch.batch.swap(0, 1);
        let block_1_number = batch.batch[2].get_block_number().unwrap();
        let block_2_number = batch.batch[1].get_block_number().unwrap();
        let expected_error = BatchingError::UnchainedBlocks {
            block_num: block_1_number,
            parent_block_num: block_2_number,
        };
        match batch.check_is_chained() {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(SentinelError::Batching(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {e}"),
        }
    }

    #[test]
    fn should_get_and_set_block_num() {
        let mut batch = Batch::default();
        assert_eq!(batch.get_block_num(), 0);
        let n = 1337;
        batch.set_block_num(n);
        assert_eq!(batch.get_block_num(), n);
    }

    #[test]
    fn should_increment_block_num() {
        let mut batch = Batch::default();
        assert_eq!(batch.get_block_num(), 0);
        batch.increment_block_num();
        assert_eq!(batch.get_block_num(), 1);
    }
}
