use std::{result::Result, str::FromStr};

use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_eth::convert_hex_to_eth_address;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use crate::{config::ConfigT, constants::MILLISECONDS_MULTIPLIER, Endpoints, SentinelError};

#[derive(Debug, Clone, Deserialize)]
pub struct NativeToml {
    validate: bool,
    router: String,
    sleep_duration: u64,
    eth_chain_id: String,
    state_manager: String,
    endpoints: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct NativeConfig {
    validate: bool,
    router: EthAddress,
    sleep_duration: u64,
    endpoints: Endpoints,
    eth_chain_id: EthChainId,
    state_manager: EthAddress,
}

impl NativeConfig {
    pub fn from_toml(toml: &NativeToml) -> Result<Self, SentinelError> {
        let sleep_duration = toml.sleep_duration * MILLISECONDS_MULTIPLIER;
        Ok(Self {
            sleep_duration,
            validate: toml.validate,
            router: convert_hex_to_eth_address(&toml.router)?,
            state_manager: convert_hex_to_eth_address(&toml.state_manager)?,
            endpoints: Endpoints::new(false, sleep_duration, BridgeSide::Native, toml.endpoints.clone()),
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

    fn state_manager(&self) -> EthAddress {
        self.state_manager
    }

    fn router(&self) -> EthAddress {
        self.router
    }
}
