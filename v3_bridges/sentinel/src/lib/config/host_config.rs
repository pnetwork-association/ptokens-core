use std::str::FromStr;

use anyhow::Result;
use common_metadata::MetadataChainId;
use serde::Deserialize;

use crate::config::Endpoints;

#[derive(Debug, Clone, Deserialize)]
pub struct HostToml {
    sleep_time: u64,
    chain_id: String,
    endpoints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HostConfig {
    sleep_time: u64,
    endpoints: Endpoints,
    chain_id: MetadataChainId,
}

impl HostConfig {
    pub fn from_toml(toml: &HostToml) -> Self {
        Self {
            sleep_time: toml.sleep_time,
            endpoints: Endpoints::new(false, toml.endpoints.clone()),
            chain_id: match MetadataChainId::from_str(&toml.chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `host_chain_id` from config, defaulting to `EthereumMainnet`");
                    warn!("{e}");
                    MetadataChainId::EthereumMainnet
                },
            },
        }
    }

    pub fn get_first_endpoint(&self) -> Result<String> {
        info!("Getting first host endpoint");
        self.endpoints.get_first_endpoint()
    }

    pub fn get_endpoints(&self) -> Endpoints {
        info!("Getting host endpoints!");
        self.endpoints.clone()
    }
}
