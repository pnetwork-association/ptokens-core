use std::{result::Result, time::SystemTime};

use common_eth::{EthSubmissionMaterial, EthSubmissionMaterials};
use ethereum_types::{Address as EthAddress, U256};
use jsonrpsee::ws_client::WsClient;

use crate::{config::Config, endpoints::Endpoints, SentinelError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Batch {
    is_native: bool,
    batch_size: u64,
    sleep_duration: u64,
    batch_duration: u64,
    endpoints: Endpoints,
    batching_is_disabled: bool,
    last_submitted: SystemTime,
    batch: EthSubmissionMaterials,
    contract_addresses: Vec<EthAddress>,
}

impl Default for Batch {
    fn default() -> Self {
        Self {
            batch_size: 1,
            is_native: true,
            sleep_duration: 0,
            batch_duration: 300, // NOTE: 5mins
            contract_addresses: vec![],
            batching_is_disabled: false,
            endpoints: Endpoints::default(),
            last_submitted: SystemTime::now(),
            batch: EthSubmissionMaterials::new(vec![]),
        }
    }
}

impl Batch {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_rpc_client(&self) -> Result<WsClient, SentinelError> {
        self.endpoints.get_rpc_client().await
    }

    pub fn is_native(&self) -> bool {
        self.is_native
    }

    pub fn is_host(&self) -> bool {
        !self.is_native
    }

    pub fn get_sleep_duration(&self) -> u64 {
        self.sleep_duration
    }

    pub fn new_from_config(is_native: bool, config: &Config) -> Result<Self, SentinelError> {
        info!("Getting Batch from config...");
        let res = Self {
            is_native,
            sleep_duration: if is_native {
                config.native_config.get_sleep_duration()
            } else {
                config.host_config.get_sleep_duration()
            },
            endpoints: if is_native {
                config.native_config.get_endpoints()
            } else {
                config.host_config.get_endpoints()
            },
            batch_size: config.batching_config.get_batch_size(is_native),
            batch_duration: config.batching_config.get_batch_duration(is_native),
            contract_addresses: if is_native {
                config.native_config.get_contract_addresses()
            } else {
                config.host_config.get_contract_addresses()
            },
            ..Default::default()
        };
        if res.endpoints.is_empty() {
            Err(SentinelError::Batching(Error::NoEndpoint(is_native)))
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
        self.last_submitted = SystemTime::now();
    }

    pub fn get_time_of_last_submission(&self) -> SystemTime {
        self.last_submitted
    }

    pub fn push(&mut self, sub_mat: EthSubmissionMaterial) {
        self.batch
            .push(sub_mat.remove_receipts_if_no_logs_from_addresses(&self.contract_addresses));
    }

    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    pub fn drain(&mut self) {
        self.batch = EthSubmissionMaterials::new(vec![]);
        self.set_time_of_last_submission()
    }

    pub fn size_in_blocks(&self) -> u64 {
        self.batch.len() as u64
    }

    pub fn is_ready_to_submit(&self) -> bool {
        if self.is_empty() {
            // NOTE: There's nothing to submit.
            return false;
        } else if self.size_in_blocks() >= self.batch_size {
            // NOTE: We've reached the max allowable batch size for submissions...
            info!(
                "[+] Ready to submit because batch has sufficient blocks! (Num blocks: {}, limit: {})",
                self.size_in_blocks(),
                self.batch_size
            );
            return true;
        }
        if let Ok(t) = self.last_submitted.elapsed() {
            let res = t.as_secs() >= self.batch_duration;
            if res {
                info!("[+] Ready to submit because enough time has elapsed");
                true
            } else {
                false
            }
        } else {
            // NOTE: If there's some error figuring out the elapsed time, let's assume it's ready...
            warn!("[!] Could not ascertain elapsed time since last submission, so assuming it's ready!");
            true
        }
    }

    pub fn check_is_chained(self) -> Result<Self, SentinelError> {
        let num_blocks_in_batch = self.size_in_blocks() as usize;
        if num_blocks_in_batch < 2 {
            info!("No need to check batch chaining - it contains too few blocks to matter!");
            Ok(self)
        } else {
            info!("Checking batch is chained correctly...");
            let mut i = num_blocks_in_batch - 1;
            while i > 0 {
                if self.batch[i].get_parent_hash()? != self.batch[i - 1].get_block_hash()? {
                    let n_1 = self.batch[i].get_block_number()?;
                    let n_2 = self.batch[i - 1].get_block_number()?;
                    return Err(SentinelError::Batching(Error::UnchainedBlocks {
                        block_num: n_1,
                        parent_block_num: n_2,
                    }));
                }
                i -= 1;
            }
            info!("Batch is chained correctly!");
            Ok(self)
        }
    }

    pub fn to_submission_material(&self) -> EthSubmissionMaterials {
        self.batch.clone()
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Two blocks in the batch whose parent_hash & hash do not match.
    UnchainedBlocks { block_num: U256, parent_block_num: U256 },

    /// No endpoint error
    NoEndpoint(bool),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::UnchainedBlocks {
                block_num: ref b,
                parent_block_num: ref p,
            } => write!(f, "block num {b} is not chained correctly to {p}"),
            Self::NoEndpoint(ref is_native) => write!(
                f,
                "Cannot create {} sub mat batch - no endpoints!",
                if is_native == &true { "native" } else { "host" },
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;

        match self {
            UnchainedBlocks { .. } | NoEndpoint(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use common_eth::{convert_hex_to_eth_address, EthLog, EthLogs, EthReceipt, EthReceipts};

    use super::*;
    use crate::test_utils::get_sample_batch;

    #[test]
    fn should_enable_batching() {
        let mut batch = Batch::new();
        batch.disable_batching();
        assert!(!batch.batching_is_enabled());
        batch.enable_batching();
        let result = batch.batching_is_enabled();
        assert!(result);
    }

    #[test]
    fn should_disable_batching() {
        let mut batch = Batch::new();
        batch.disable_batching();
        let result = batch.batching_is_enabled();
        assert!(!result);
    }

    #[test]
    fn should_set_time_of_last_submission() {
        let mut batch = Batch::new();
        let timestamp_before = batch.get_time_of_last_submission();
        batch.set_time_of_last_submission();
        let result = batch.get_time_of_last_submission();
        assert!(result > timestamp_before);
    }

    #[test]
    fn should_push_to_batch() {
        let mut batch = Batch::new();
        assert!(batch.is_empty());
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert!(!batch.is_empty());
    }

    #[test]
    fn should_drain_batch() {
        let mut batch = Batch::new();
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert!(!batch.is_empty());
        batch.drain();
        assert!(batch.is_empty());
    }

    #[test]
    fn should_get_size_in_blocks_of_batch() {
        let mut batch = Batch::new();
        assert_eq!(batch.size_in_blocks(), 0);
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert_eq!(batch.size_in_blocks(), 1);
    }

    #[test]
    fn pushed_block_should_have_receipts_if_they_contain_pertinent_logs() {
        let address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let mut log = EthLog::default();
        log.address = address;
        let logs = EthLogs::new(vec![log]);
        let mut receipt = EthReceipt::default();
        receipt.logs = logs;
        let receipts = EthReceipts::new(vec![receipt]);
        let mut sub_mat = EthSubmissionMaterial::default();
        sub_mat.receipts = receipts.clone();
        let mut batch = Batch::new();
        batch.contract_addresses = vec![address];
        batch.push(sub_mat);
        assert_eq!(batch.batch[0].receipts, receipts);
    }

    #[test]
    fn pushed_block_should_not_have_receipts_if_they_contain_pertinent_logs() {
        let address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let other_address = convert_hex_to_eth_address("0x690b9a9e9aa1c9db991c7721a92d351db4fac990").unwrap();
        let mut log = EthLog::default();
        log.address = address;
        let logs = EthLogs::new(vec![log]);
        let mut receipt = EthReceipt::default();
        receipt.logs = logs;
        let receipts = EthReceipts::new(vec![receipt]);
        let mut sub_mat = EthSubmissionMaterial::default();
        sub_mat.receipts = receipts.clone();
        let mut batch = Batch::new();
        batch.contract_addresses = vec![other_address];
        batch.push(sub_mat);
        assert!(batch.batch[0].receipts.is_empty());
    }

    #[test]
    fn should_pass_is_chained_check_if_batch_is_empty() {
        let batch = Batch::new();
        assert!(batch.check_is_chained().is_ok())
    }

    #[test]
    fn should_pass_is_chained_check_if_batch_has_one_member() {
        let mut batch = Batch::new();
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
        let expected_error = Error::UnchainedBlocks {
            block_num: block_1_number,
            parent_block_num: block_2_number,
        };
        match batch.check_is_chained() {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(SentinelError::Batching(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {e}"),
        }
    }
}