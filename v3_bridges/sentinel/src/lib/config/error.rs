use common::BridgeSide;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("logs of size {size}b is not between min of {min}b and max of {max}b ")]
    LogSize { size: u64, min: u64, max: u64 },

    #[error("number of logs of {size} is not between min of {min} and max of {max}")]
    LogNum { size: usize, min: usize, max: usize },

    #[error("batch duration of {size} is greater than max of {max}")]
    BatchDuration { size: u64, max: u64 },

    #[error("batch size of {size} is not between min of {min} and max of {max}")]
    BatchSize { size: u64, min: u64, max: u64 },

    #[error("Cannot create {0} sub mat batch - there are  no endpoints")]
    NoEndpoints(BridgeSide),
}
