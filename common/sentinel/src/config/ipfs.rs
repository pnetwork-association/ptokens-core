use derive_getters::Getters;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Getters, Deserialize)]
pub struct IpfsConfig {
    ipfs_bin_path: String,
    status_update_frequency: u64,
}
