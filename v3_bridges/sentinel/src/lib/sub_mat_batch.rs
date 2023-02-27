use std::time::{Duration, SystemTime};

use common_eth::EthSubmissionMaterial;

use crate::Config;

#[derive(Debug, Eq, PartialEq)]
pub struct SubMatBatch {
    batching_is_disabled: bool,
    last_submitted: SystemTime,
    batch: Vec<EthSubmissionMaterial>,
}

impl Default for SubMatBatch {
    fn default() -> Self {
        Self {
            batch: vec![],
            batching_is_disabled: false,
            last_submitted: SystemTime::now(),
        }
    }
}

impl SubMatBatch {
    pub fn new() -> Self {
        Self {
            batch: vec![],
            batching_is_disabled: false,
            last_submitted: SystemTime::now(),
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

    pub fn size_in_blocks(&self) -> usize {
        self.batch.len()
    }

    pub fn is_ready_to_submit(&self, config: &Config, is_native: bool) -> bool {
        if self.is_empty() {
            // NOTE: There's nothing to submit.
            return false;
        } else if self.size_in_blocks() >= config.batching.get_batch_size(is_native) {
            // NOTE: We've reached the max allowable batch size for submissions...
            return true;
        }
        if let Ok(t) = self.last_submitted.elapsed() {
            return t.as_secs() >= config.batching.get_batch_duration(is_native) as u64;
        } else {
            // NOTE: If there's some error figuring out the elapsed time, let's assume it's ready...
            warn!("Could not ascertain elapsed time since last submission, so assuming it's ready!");
            return true;
        }
        // NOTE: Can't think of anything else to check, so let's assume it's ready by default.
        return true;
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
