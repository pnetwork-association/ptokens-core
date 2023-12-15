use thiserror::Error;

use crate::NetworkId;

#[derive(Error, Debug)]
pub enum SentinelConfigError {
    #[error("sentinel config network id error {0}")]
    NetworkId(#[from] crate::NetworkIdError),

    #[error("logs of size {size}b is not between min of {min}b and max of {max}b ")]
    LogSize { size: u64, min: u64, max: u64 },

    #[error("number of logs of {size} is not between min of {min} and max of {max}")]
    LogNum { size: usize, min: usize, max: usize },

    #[error("batch duration of {size} is greater than max of {max}")]
    BatchDuration { size: u64, max: u64 },

    #[error("base challenge period duration of {size} is not between min of {min} and max of {max}")]
    BaseChallengePeriodDuration { size: u64, min: u64, max: u64 },

    #[error("batch size of {size} is not between min of {min} and max of {max}")]
    BatchSize { size: u64, min: u64, max: u64 },

    #[error("Cannot create sub mat batch for network {0} - there are  no endpoints")]
    NoEndpoints(NetworkId),

    #[error("no config for network id {0}")]
    NoConfig(NetworkId),
}
