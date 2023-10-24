use std::result::Result;

use common_chain_ids::EthChainId;
use derive_getters::Getters;
use ethereum_types::Address as EthAddress;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::{
    config::{
        GovernanceConfig,
        GovernanceToml,
        HostConfig,
        HostToml,
        IpfsConfig,
        LogConfig,
        LogToml,
        NativeConfig,
        NativeToml,
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
    host: HostToml,
    ipfs: IpfsConfig,
    native: NativeToml,
    core: SentinelCoreConfig,
    governance: GovernanceToml,
}

impl SentinelConfigToml {
    pub fn new(path: &str) -> Result<Self, SentinelError> {
        Ok(config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()?
            .try_deserialize()?)
    }
}

#[derive(Debug, Clone, Getters)]
pub struct SentinelConfig {
    log: LogConfig,
    host: HostConfig,
    ipfs: IpfsConfig,
    native: NativeConfig,
    core: SentinelCoreConfig,
    governance: GovernanceConfig,
}

impl SentinelConfig {
    pub fn new(path: &str) -> Result<Self, SentinelError> {
        let res = Self::from_toml(&SentinelConfigToml::new(path)?)?;
        debug!("sentinel config {:?}", res);
        Ok(res)
    }

    fn from_toml(toml: &SentinelConfigToml) -> Result<Self, SentinelError> {
        Ok(Self {
            ipfs: toml.ipfs.clone(),
            core: toml.core.clone(),
            log: LogConfig::from_toml(&toml.log)?,
            host: HostConfig::from_toml(&toml.host)?,
            native: NativeConfig::from_toml(&toml.native)?,
            governance: GovernanceConfig::try_from(&toml.governance)?,
        })
    }

    pub fn get_log_level(&self) -> LogLevel {
        self.log.level
    }

    pub fn get_host_endpoints(&self) -> Endpoints {
        self.host.endpoints()
    }

    pub fn get_native_endpoints(&self) -> Endpoints {
        self.native.endpoints()
    }

    pub fn endpoints(&self, nid: &NetworkId) -> Result<Endpoints, SentinelConfigError> {
        if self.native().network_id() == nid {
            Ok(self.native().endpoints())
        } else if self.host().network_id() == nid {
            Ok(self.host().endpoints())
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }

    pub fn validate(&self, nid: &NetworkId) -> Result<bool, SentinelConfigError> {
        if self.native().network_id() == nid {
            Ok(*self.native().validate())
        } else if self.host().network_id() == nid {
            Ok(*self.host().validate())
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }

    pub fn pnetwork_hub(&self, nid: &NetworkId) -> Result<EthAddress, SentinelConfigError> {
        if self.native().network_id() == nid {
            Ok(*self.native().pnetwork_hub())
        } else if self.host().network_id() == nid {
            Ok(*self.host().pnetwork_hub())
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }

    pub fn gas_price(&self, network_id: &NetworkId) -> Option<u64> {
        if self.native().network_id() == network_id {
            *self.native().gas_price()
        } else if self.host().network_id() == network_id {
            *self.host().gas_price()
        } else {
            None
        }
    }

    pub fn gas_limit(&self, nid: &NetworkId) -> Result<usize, SentinelConfigError> {
        if self.native().network_id() == nid {
            Ok(*self.native().gas_limit())
        } else if self.host().network_id() == nid {
            Ok(*self.host().gas_limit())
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }

    pub fn network_ids(&self) -> Result<Vec<NetworkId>, SentinelConfigError> {
        Ok(vec![*self.native().network_id(), *self.host().network_id()])
    }

    pub fn governance_address(&self, nid: &NetworkId) -> Option<EthAddress> {
        // NOTE: The governance contract lives on one chain only
        if nid == self.governance().network_id() {
            Some(*self.governance().governance_address())
        } else {
            None
        }
    }

    pub fn eth_chain_id(&self, nid: &NetworkId) -> Result<EthChainId, SentinelConfigError> {
        let n_network_id = self.native().network_id();
        let h_network_id = self.host().network_id();

        if n_network_id == nid {
            Ok(EthChainId::try_from(n_network_id)?)
        } else if h_network_id == nid {
            Ok(EthChainId::try_from(h_network_id)?)
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }

    pub fn pre_filter_receipts(&self, nid: &NetworkId) -> Result<bool, SentinelConfigError> {
        if self.native().network_id() == nid {
            Ok(*self.native().pre_filter_receipts())
        } else if self.host().network_id() == nid {
            Ok(*self.host().pre_filter_receipts())
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }

    pub fn sleep_duration(&self, nid: &NetworkId) -> Result<u64, SentinelConfigError> {
        if self.native().network_id() == nid {
            Ok(*self.native().sleep_duration())
        } else if self.host().network_id() == nid {
            Ok(*self.host().sleep_duration())
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }

    pub fn batch_duration(&self, nid: &NetworkId) -> Result<u64, SentinelConfigError> {
        if self.native().network_id() == nid {
            Ok(*self.native().batch_duration())
        } else if self.host().network_id() == nid {
            Ok(*self.host().batch_duration())
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }

    pub fn batch_size(&self, nid: &NetworkId) -> Result<u64, SentinelConfigError> {
        if self.native().network_id() == nid {
            Ok(*self.native().batch_size())
        } else if self.host().network_id() == nid {
            Ok(*self.host().batch_size())
        } else {
            Err(SentinelConfigError::NoConfig(*nid))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_config() {
        let path = "src/config/test_utils/sample-config";
        let result = SentinelConfig::new(path);
        result.unwrap();
    }
}
