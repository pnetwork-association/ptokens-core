use derive_getters::Getters;
use serde::Deserialize;


#[derive(Debug, Default, Clone, Getters, Deserialize)]
pub struct SentinelCoreConfig {
    timeout: u64,
    max_cancellable_time_delta: u64,
    challenge_response_frequency: u64,
}
