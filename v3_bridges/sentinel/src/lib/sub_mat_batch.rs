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
}
