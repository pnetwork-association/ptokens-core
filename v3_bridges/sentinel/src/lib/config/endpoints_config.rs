use std::str::FromStr;

use anyhow::Result;
use common_metadata::MetadataChainId;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::errors::SentinelError;

#[derive(Debug, Clone, Deserialize)]
pub struct EndpointsToml {
    host: Vec<String>,
    native: Vec<String>,
    host_chain_id: String,
    native_chain_id: String,
}

#[derive(Debug, Clone)]
pub struct EndpointsConfig {
    host: Vec<String>,
    native: Vec<String>,
    host_chain_id: MetadataChainId,
    native_chain_id: MetadataChainId,
}

impl EndpointsConfig {
    pub fn from_toml(toml: &EndpointsToml) -> Self {
        Self {
            host: toml.host.clone(),
            native: toml.native.clone(),
            host_chain_id: match MetadataChainId::from_str(&toml.host_chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `host_chain_id` from config, defaulting to `EthereumMainnet`");
                    warn!("{e}");
                    MetadataChainId::EthereumMainnet
                },
            },
            native_chain_id: match MetadataChainId::from_str(&toml.native_chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `native_chain_id` from config, defaulting to `EthereumMainnet`");
                    warn!("{e}");
                    MetadataChainId::EthereumMainnet
                },
            },
        }
    }

    pub fn get_first_endpoint(&self, is_native: bool) -> Result<String> {
        let endpoint_type = if is_native { "native" } else { "host" };
        info!("Getting first {endpoint_type} endpoint...");
        let err = format!("No {endpoint_type} endpoints in config file!");
        if is_native {
            if self.native.is_empty() {
                Err(anyhow!(err))
            } else {
                Ok(self.native[0].clone())
            }
        } else if self.host.is_empty() {
            Err(anyhow!(err))
        } else {
            Ok(self.host[0].clone())
        }
    }
}
