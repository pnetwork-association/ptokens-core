use common_eth::{convert_hex_to_eth_address, convert_hex_to_h256};
use common_network_ids::NetworkId;
use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use serde::{Deserialize, Serialize};

use super::SentinelConfigError;
use crate::{Endpoints, SentinelError};

#[derive(Debug, Clone, Default, Getters, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct ConfiguredEvent {
    pub address: EthAddress,
    pub topic: EthHash,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize, Constructor, Deref)]
pub struct ConfiguredEvents(Vec<ConfiguredEvent>);

impl TryFrom<&Vec<Vec<String>>> for ConfiguredEvents {
    type Error = SentinelConfigError;

    fn try_from(e: &Vec<Vec<String>>) -> Result<Self, Self::Error> {
        let events = e
            .iter()
            .map(|v| {
                if v.len() < 2 {
                    Err(Self::Error::NotEnoughEventArgs)
                } else {
                    Ok(ConfiguredEvent::new(
                        convert_hex_to_eth_address(&v[0])?,
                        convert_hex_to_h256(&v[1])?,
                    ))
                }
            })
            .collect::<Result<Vec<_>, Self::Error>>()?;

        Ok(Self::new(events))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkToml {
    validate: bool,
    batch_size: u64,
    gas_limit: usize,
    batch_duration: u64,
    sleep_duration: u64,
    pnetwork_hub: String,
    endpoints: Vec<String>,
    gas_price: Option<u64>,
    events: Vec<Vec<String>>,
    pre_filter_receipts: bool,
}

#[derive(Debug, Clone, Default, Getters, Eq, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfig {
    validate: bool,
    batch_size: u64,
    gas_limit: usize,
    batch_duration: u64,
    sleep_duration: u64,
    #[getter(skip)]
    endpoints: Endpoints,
    gas_price: Option<u64>,
    pnetwork_hub: EthAddress,
    events: ConfiguredEvents,
    pre_filter_receipts: bool,
}

impl NetworkConfig {
    pub fn network_id(&self) -> NetworkId {
        *self.endpoints.network_id()
    }

    pub fn from_toml(network_id: NetworkId, toml: &NetworkToml) -> Result<Self, SentinelError> {
        let sleep_duration = toml.sleep_duration;
        let endpoints = Endpoints::new(sleep_duration, network_id, toml.endpoints.clone());
        Ok(Self {
            endpoints,
            sleep_duration,
            validate: toml.validate,
            gas_price: toml.gas_price,
            gas_limit: toml.gas_limit,
            events: ConfiguredEvents::try_from(&toml.events)?,
            pre_filter_receipts: toml.pre_filter_receipts,
            batch_size: Self::sanity_check_batch_size(toml.batch_size)?,
            pnetwork_hub: convert_hex_to_eth_address(&toml.pnetwork_hub)?,
            batch_duration: Self::sanity_check_batch_duration(toml.batch_duration)?,
        })
    }

    pub fn endpoints(&self) -> Endpoints {
        self.endpoints.clone()
    }

    pub fn get_sleep_duration(&self) -> u64 {
        self.sleep_duration
    }

    fn sanity_check_batch_size(batch_size: u64) -> Result<u64, SentinelError> {
        info!("sanity checking batch size...");
        const MIN: u64 = 0;
        const MAX: u64 = 1000;
        if batch_size > MIN && batch_size <= MAX {
            Ok(batch_size)
        } else {
            Err(SentinelError::SentinelConfig(SentinelConfigError::BatchSize {
                size: batch_size,
                min: MIN,
                max: MAX,
            }))
        }
    }

    fn sanity_check_batch_duration(batch_duration: u64) -> Result<u64, SentinelError> {
        info!("sanity checking batch duration...");
        // NOTE: A batch duration of 0 means we submit material one at a time...
        const MAX: u64 = 60 * 10; // NOTE: Ten mins
        if batch_duration <= MAX {
            Ok(batch_duration)
        } else {
            Err(SentinelError::SentinelConfig(SentinelConfigError::BatchDuration {
                max: MAX,
                size: batch_duration,
            }))
        }
    }
}
