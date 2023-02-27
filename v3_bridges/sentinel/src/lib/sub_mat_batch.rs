use std::time::{Duration, SystemTime};

use common_eth::EthSubmissionMaterial;

#[derive(Debug, Eq, PartialEq)]
pub struct SubMatBatch {
    batching_is_disabled: bool,
    last_submitted: SystemTime,
    batch: Vec<EthSubmissionMaterial>,
}

impl SubMatBatch {
    pub fn new() -> Self {
        Self {
            batch: vec![],
            batching_is_disabled: false,
            last_submitted: SystemTime::now(),
        }
    }

    pub fn enable_batching(mut self) -> Self {
        self.batching_is_disabled = false;
        self
    }

    pub fn disable_batching(mut self) -> Self {
        self.batching_is_disabled = true;
        self
    }

    pub fn batching_is_enabled(&self) -> bool {
        !self.batching_is_disabled
    }

    pub fn set_time_of_last_submission(mut self) -> Self {
        self.last_submitted = SystemTime::now();
        self
    }

    pub fn get_time_of_last_submission(&self) -> SystemTime {
        self.last_submitted
    }

    pub fn push(mut self, sub_mat: EthSubmissionMaterial) -> Self {
        self.batch.push(sub_mat);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    pub fn drain(mut self) -> Self {
        self.batch = vec![];
        self.set_time_of_last_submission()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_enable_batching() {
        let batch = SubMatBatch::new();
        let updated_batch = batch.disable_batching();
        assert!(!updated_batch.batching_is_enabled());
        let result = updated_batch.enable_batching().batching_is_enabled();
        assert!(result);
    }

    #[test]
    fn should_disable_batching() {
        let batch = SubMatBatch::new();
        let result = batch.disable_batching().batching_is_enabled();
        assert!(!result);
    }

    #[test]
    fn should_set_time_of_last_submission() {
        let batch = SubMatBatch::new();
        let timestamp_before = batch.get_time_of_last_submission();
        let updated_batch = batch.set_time_of_last_submission();
        let result = updated_batch.get_time_of_last_submission();
        assert!(result > timestamp_before);
    }

    #[test]
    fn should_push_to_batch() {
        let batch = SubMatBatch::new();
        assert!(batch.is_empty());
        let sub_mat = EthSubmissionMaterial::default();
        let updated_batch = batch.push(sub_mat);
        assert!(!updated_batch.is_empty());
    }

    #[test]
    fn should_drain_batch() {
        let batch = SubMatBatch::new();
        let sub_mat = EthSubmissionMaterial::default();
        let updated_batch = batch.push(sub_mat);
        assert!(!updated_batch.is_empty());
        let final_batch = updated_batch.drain();
        assert!(final_batch.is_empty());
    }
}
