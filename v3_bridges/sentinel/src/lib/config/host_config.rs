use std::{result::Result, str::FromStr};

use common::BridgeSide;
use common_chain_ids::EthChainId;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use crate::{
    config::{ConfigT, ContractInfoToml, ContractInfos},
    constants::MILLISECONDS_MULTIPLIER,
    Endpoints,
    SentinelError,
};

#[derive(Debug, Clone, Deserialize)]
pub struct HostToml {
    sleep_duration: u64,
    eth_chain_id: String,
    endpoints: Vec<String>,
    contract_info: Vec<ContractInfoToml>,
}

#[derive(Debug, Clone)]
pub struct HostConfig {
    sleep_duration: u64,
    endpoints: Endpoints,
    eth_chain_id: EthChainId,
    contract_infos: ContractInfos,
}

impl HostConfig {
    pub fn from_toml(toml: &HostToml) -> Result<Self, SentinelError> {
        let sleep_duration = toml.sleep_duration * MILLISECONDS_MULTIPLIER;
        Ok(Self {
            sleep_duration,
            endpoints: Endpoints::new(false, sleep_duration, toml.endpoints.clone()),
            contract_infos: ContractInfos::from_tomls(&toml.contract_info)?,
            eth_chain_id: match EthChainId::from_str(&toml.eth_chain_id) {
                Ok(id) => id,
                Err(e) => {
                    warn!("Could not parse `eth_chain_id` from host config, defaulting to ETH mainnet!");
                    warn!("{e}");
                    EthChainId::Mainnet
                },
            },
        })
    }

    pub fn get_first_endpoint(&self) -> Result<String, SentinelError> {
        info!("Getting first host endpoint");
        self.endpoints.get_first_endpoint()
    }

    pub fn get_endpoints(&self) -> Endpoints {
        info!("Getting host endpoints!");
        self.endpoints.clone()
    }

    pub fn get_sleep_duration(&self) -> u64 {
        self.sleep_duration
    }

    pub fn get_eth_chain_id(&self) -> EthChainId {
        self.eth_chain_id.clone()
    }
}

impl ConfigT for HostConfig {
    fn get_contract_addresses(&self) -> Vec<EthAddress> {
        self.contract_infos.get_addresses()
    }

    fn side(&self) -> BridgeSide {
        BridgeSide::Host
    }
}
