use std::str::FromStr;

use anyhow::Result;
use common_eth::convert_hex_strings_to_eth_addresses;
use common_metadata::MetadataChainId;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use crate::{config::Endpoints, constants::MILLISECONDS_MULTIPLIER};

#[derive(Debug, Clone, Deserialize)]
pub struct HostToml {
    sleep_time: u64,
    chain_id: String,
    endpoints: Vec<String>,
    contract_addresses: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HostConfig {
    sleep_time: u64,
    endpoints: Endpoints,
    chain_id: MetadataChainId,
    contract_addresses: Vec<EthAddress>,
}

impl HostConfig {
    pub fn from_toml(toml: &HostToml) -> Result<Self> {
        let sleep_time = toml.sleep_time * MILLISECONDS_MULTIPLIER;
        Ok(Self {
            sleep_time,
            endpoints: Endpoints::new(false, sleep_time, toml.endpoints.clone()),
            contract_addresses: convert_hex_strings_to_eth_addresses(&toml.contract_addresses)?,
            chain_id: match MetadataChainId::from_str(&toml.chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `host_chain_id` from config, defaulting to `EthereumMainnet`");
                    warn!("{e}");
                    MetadataChainId::EthereumMainnet
                },
            },
        })
    }

    pub fn get_first_endpoint(&self) -> Result<String> {
        info!("Getting first host endpoint");
        self.endpoints.get_first_endpoint()
    }

    pub fn get_endpoints(&self) -> Endpoints {
        info!("Getting host endpoints!");
        self.endpoints.clone()
    }

    pub fn get_contract_addresses(&self) -> Vec<EthAddress> {
        self.contract_addresses.clone()
    }
}
