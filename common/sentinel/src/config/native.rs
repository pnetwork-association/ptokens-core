use common::BridgeSide;
use common_eth::convert_hex_to_eth_address;
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use super::SentinelConfigError;
use crate::{config::ConfigT, constants::MILLISECONDS_MULTIPLIER, Endpoints, NetworkId, SentinelError};

#[derive(Debug, Clone, Deserialize)]
pub struct NativeToml {
    validate: bool,
    gas_limit: usize,
    network_id: String,
    sleep_duration: u64,
    pnetwork_hub: String,
    endpoints: Vec<String>,
    gas_price: Option<u64>,
    pre_filter_receipts: bool,
}

#[derive(Debug, Clone, Default, Getters)]
pub struct NativeConfig {
    validate: bool,
    gas_limit: usize,
    sleep_duration: u64,
    #[getter(skip)]
    endpoints: Endpoints,
    network_id: NetworkId,
    gas_price: Option<u64>,
    pnetwork_hub: EthAddress,
    pre_filter_receipts: bool,
}

impl NativeConfig {
    pub fn from_toml(toml: &NativeToml) -> Result<Self, SentinelError> {
        let sleep_duration = toml.sleep_duration * MILLISECONDS_MULTIPLIER;
        Ok(Self {
            sleep_duration,
            validate: toml.validate,
            gas_price: toml.gas_price,
            gas_limit: toml.gas_limit,
            pre_filter_receipts: toml.pre_filter_receipts,
            network_id: NetworkId::try_from(&toml.network_id)?,
            pnetwork_hub: convert_hex_to_eth_address(&toml.pnetwork_hub)?,
            endpoints: Endpoints::new(sleep_duration, BridgeSide::Native, toml.endpoints.clone()),
        })
    }

    pub fn endpoints(&self) -> Endpoints {
        self.endpoints.clone()
    }

    pub fn get_sleep_duration(&self) -> u64 {
        self.sleep_duration
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

    fn metadata_chain_id(&self) -> Result<MetadataChainId, SentinelConfigError> {
        Ok(MetadataChainId::try_from(self.network_id())?)
    }

    fn mcid(&self) -> Result<MetadataChainId, SentinelConfigError> {
        self.metadata_chain_id()
    }

    fn pre_filter_receipts(&self) -> bool {
        *self.pre_filter_receipts()
    }
}
