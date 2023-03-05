use std::time::SystemTime;

use anyhow::Result;
use common_eth::EthSubmissionMaterial;
use jsonrpsee::ws_client::WsClient;

use crate::config::{Config, Endpoints};

#[derive(Debug, Eq, PartialEq)]
pub struct SubMatBatch {
    is_native: bool,
    batch_size: u64,
    batch_duration: u64,
    endpoints: Endpoints,
    batching_is_disabled: bool,
    last_submitted: SystemTime,
    batch: Vec<EthSubmissionMaterial>,
}

impl Default for SubMatBatch {
    fn default() -> Self {
        Self {
            batch: vec![],
            batch_size: 1,
            is_native: true,
            batch_duration: 300, // NOTE: 5mins
            batching_is_disabled: false,
            endpoints: Endpoints::default(),
            last_submitted: SystemTime::now(),
        }
    }
}

impl SubMatBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_rpc_client(&self) -> Result<WsClient> {
        Ok(self.endpoints.get_rpc_client().await?)
    }

    pub fn is_native(&self) -> bool {
        self.is_native
    }

    pub fn is_host(&self) -> bool {
        !self.is_native
    }

    pub fn new_from_config(is_native: bool, config: &Config) -> Result<Self> {
        let res = Self {
            is_native,
            endpoints: if is_native {
                config.native_config.get_endpoints()
            } else {
                config.host_config.get_endpoints()
            },
            batch_size: config.batching_config.get_batch_size(is_native),
            batch_duration: config.batching_config.get_batch_duration(is_native),
            ..Default::default()
        };
        if res.endpoints.is_empty() {
            Err(anyhow!(format!(
                "Cannot create {} sub mat batch - no endpoints!",
                if is_native { "native" } else { "host" }
            )))
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
        self.batch.push(sub_mat);
    }

    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    pub fn drain(&mut self) {
        self.batch = vec![];
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
                return true;
            } else {
                return false;
            }
        } else {
            // NOTE: If there's some error figuring out the elapsed time, let's assume it's ready...
            warn!("[!] Could not ascertain elapsed time since last submission, so assuming it's ready!");
            return true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_enable_batching() {
        let mut batch = SubMatBatch::new();
        batch.disable_batching();
        assert!(!batch.batching_is_enabled());
        batch.enable_batching();
        let result = batch.batching_is_enabled();
        assert!(result);
    }

    #[test]
    fn should_disable_batching() {
        let mut batch = SubMatBatch::new();
        batch.disable_batching();
        let result = batch.batching_is_enabled();
        assert!(!result);
    }

    #[test]
    fn should_set_time_of_last_submission() {
        let mut batch = SubMatBatch::new();
        let timestamp_before = batch.get_time_of_last_submission();
        batch.set_time_of_last_submission();
        let result = batch.get_time_of_last_submission();
        assert!(result > timestamp_before);
    }

    #[test]
    fn should_push_to_batch() {
        let mut batch = SubMatBatch::new();
        assert!(batch.is_empty());
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert!(!batch.is_empty());
    }

    #[test]
    fn should_drain_batch() {
        let mut batch = SubMatBatch::new();
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert!(!batch.is_empty());
        batch.drain();
        assert!(batch.is_empty());
    }

    #[test]
    fn should_get_size_in_blocks_of_batch() {
        let mut batch = SubMatBatch::new();
        assert_eq!(batch.size_in_blocks(), 0);
        let sub_mat = EthSubmissionMaterial::default();
        batch.push(sub_mat);
        assert_eq!(batch.size_in_blocks(), 1);
    }
}
