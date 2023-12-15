use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Getters, Deserialize, Eq, PartialEq, Serialize)]
pub struct SentinelCoreConfig {
    timeout: u64,
    challenge_response_frequency: u64,
}
