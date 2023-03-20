use std::result::Result;

use common_eth::convert_hex_to_eth_address;
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use crate::SentinelError;

#[derive(Debug, Clone, Deref, Constructor)]
pub struct ContractInfos(Vec<ContractInfo>);

impl ContractInfos {
    pub fn from_tomls(tomls: &[ContractInfoToml]) -> Result<Self, SentinelError> {
        Ok(Self::new(
            tomls
                .iter()
                .map(ContractInfo::from_toml)
                .collect::<Result<Vec<_>, SentinelError>>()?,
        ))
    }

    pub fn get_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(ContractInfo::address).collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractInfoToml {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractInfo {
    #[allow(dead_code)]
    name: String,
    address: EthAddress,
}

impl ContractInfo {
    pub fn from_toml(toml: &ContractInfoToml) -> Result<Self, SentinelError> {
        Ok(Self {
            name: toml.name.clone(),
            address: convert_hex_to_eth_address(&toml.address)?,
        })
    }

    pub fn address(&self) -> EthAddress {
        self.address
    }
}
