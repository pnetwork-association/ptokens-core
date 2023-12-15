use std::{collections::HashMap, result::Result};

use derive_getters::Getters;
use ethereum_types::Address as EthAddress;
use log::Level as LogLevel;
use serde::{Deserialize, Serialize};

use crate::{
    config::{
        GovernanceConfig,
        GovernanceToml,
        IpfsConfig,
        LogConfig,
        LogToml,
        NetworkConfig,
        NetworkToml,
        SentinelConfigError,
        SentinelCoreConfig,
    },
    Endpoints,
    NetworkId,
    SentinelError,
};

#[derive(Debug, Clone, Deserialize)]
struct SentinelConfigToml {
    log: LogToml,
    ipfs: IpfsConfig,
    core: SentinelCoreConfig,
    governance: GovernanceToml,
    networks: HashMap<String, NetworkToml>,
}

impl SentinelConfigToml {
    pub fn new(path: &str) -> Result<Self, SentinelError> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()?
            .try_deserialize()?)
    }
}

#[derive(Debug, Clone, Getters, Eq, PartialEq, Serialize, Deserialize)]
pub struct SentinelConfig {
    log: LogConfig,
    ipfs: IpfsConfig,
    core: SentinelCoreConfig,
    governance: GovernanceConfig,
    networks: HashMap<NetworkId, NetworkConfig>,
}

impl SentinelConfig {
    pub fn new(path: &str) -> Result<Self, SentinelError> {
        let res = Self::from_toml(&SentinelConfigToml::new(path)?)?;
        debug!("sentinel config {:?}", res);
        Ok(res)
    }

    fn from_toml(toml: &SentinelConfigToml) -> Result<Self, SentinelError> {
        let mut networks: HashMap<NetworkId, NetworkConfig> = HashMap::new();
        for (k, v) in toml.networks.iter() {
            let nid = NetworkId::try_from(k)?;
            let config = NetworkConfig::from_toml(nid, v)?;
            networks.insert(nid, config);
        }

        Ok(Self {
            networks,
            ipfs: toml.ipfs.clone(),
            core: toml.core.clone(),
            log: LogConfig::from_toml(&toml.log)?,
            governance: GovernanceConfig::try_from(&toml.governance)?,
        })
    }

    pub fn log_level(&self) -> LogLevel {
        self.log.level()
    }

    pub fn endpoints(&self, nid: &NetworkId) -> Result<Endpoints, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| c.endpoints())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }

    pub fn validate(&self, nid: &NetworkId) -> Result<bool, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| *c.validate())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }

    pub fn gas_price(&self, nid: &NetworkId) -> Result<Option<u64>, SentinelConfigError> {
        let config = self
            .networks
            .get(nid)
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))?;
        Ok(*config.gas_price())
    }

    pub fn gas_limit(&self, nid: &NetworkId) -> Result<usize, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| *c.gas_limit())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }

    pub fn network_ids(&self) -> Vec<NetworkId> {
        self.networks.clone().into_keys().collect()
    }

    pub fn governance_address(&self, nid: &NetworkId) -> Option<EthAddress> {
        // NOTE: The governance contract lives on one chain only
        if nid == self.governance().network_id() {
            Some(*self.governance().governance_address())
        } else {
            None
        }
    }

    pub fn pnetwork_hub(&self, nid: &NetworkId) -> Result<EthAddress, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| *c.pnetwork_hub())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }

    pub fn pre_filter_receipts(&self, nid: &NetworkId) -> Result<bool, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| *c.pre_filter_receipts())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }

    pub fn sleep_duration(&self, nid: &NetworkId) -> Result<u64, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| *c.sleep_duration())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }

    pub fn batch_duration(&self, nid: &NetworkId) -> Result<u64, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| *c.batch_duration())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }

    pub fn batch_size(&self, nid: &NetworkId) -> Result<u64, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| *c.batch_size())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }

    pub fn base_challenge_period_duration(&self, nid: &NetworkId) -> Result<u64, SentinelConfigError> {
        self.networks
            .get(nid)
            .map(|c| *c.base_challenge_period_duration())
            .ok_or_else(|| SentinelConfigError::NoConfig(*nid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_config() {
        let path = "src/config/test_utils/sample-config";
        let result = SentinelConfig::new(path);
        assert!(result.is_ok());
    }
}
