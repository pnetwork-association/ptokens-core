use std::{result::Result, str::FromStr};

use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_eth::convert_hex_to_eth_address;
use common_metadata::MetadataChainId;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use crate::{config::ConfigT, constants::MILLISECONDS_MULTIPLIER, Endpoints, SentinelError};

#[derive(Debug, Clone, Deserialize)]
pub struct NativeToml {
    validate: bool,
    gas_limit: usize,
    sleep_duration: u64,
    eth_chain_id: String,
    pnetwork_hub: String,
    endpoints: Vec<String>,
    gas_price: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct NativeConfig {
    validate: bool,
    gas_limit: usize,
    sleep_duration: u64,
    endpoints: Endpoints,
    gas_price: Option<u64>,
    eth_chain_id: EthChainId,
    pnetwork_hub: EthAddress,
}

impl NativeConfig {
    pub fn from_toml(toml: &NativeToml) -> Result<Self, SentinelError> {
        let sleep_duration = toml.sleep_duration * MILLISECONDS_MULTIPLIER;
        Ok(Self {
            sleep_duration,
            validate: toml.validate,
            gas_price: toml.gas_price,
            gas_limit: toml.gas_limit,
            pnetwork_hub: convert_hex_to_eth_address(&toml.pnetwork_hub)?,
            endpoints: Endpoints::new(sleep_duration, BridgeSide::Native, toml.endpoints.clone()),
            eth_chain_id: match EthChainId::from_str(&toml.eth_chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `eth_chain_id` from native config, defaulting to ETH mainnet!");
                    warn!("{e}");
                    EthChainId::Mainnet
                },
            },
        })
    }

    pub fn endpoints(&self) -> Endpoints {
        self.endpoints.clone()
    }

    pub fn get_sleep_duration(&self) -> u64 {
        self.sleep_duration
    }

    pub fn get_eth_chain_id(&self) -> EthChainId {
        self.eth_chain_id.clone()
    }
}

impl ConfigT for NativeConfig {
    fn side(&self) -> BridgeSide {
        BridgeSide::Native
    }

    fn is_validating(&self) -> bool {
        self.validate
    }

    fn gas_price(&self) -> Option<u64> {
        self.gas_price
    }

    fn gas_limit(&self) -> usize {
        self.gas_limit
    }

    fn pnetwork_hub(&self) -> EthAddress {
        self.pnetwork_hub
    }

    fn chain_id(&self) -> EthChainId {
        self.eth_chain_id.clone()
    }

    fn metadata_chain_id(&self) -> MetadataChainId {
        MetadataChainId::from(&self.chain_id())
    }
}
