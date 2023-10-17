use std::result::Result;

use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use ethereum_types::Address as EthAddress;
use log::Level as LogLevel;
use serde::Deserialize;

use crate::{
    config::{
        BatchingConfig,
        BatchingToml,
        ConfigT,
        GovernanceConfig,
        GovernanceToml,
        HostConfig,
        HostToml,
        IpfsConfig,
        LogConfig,
        LogToml,
        NativeConfig,
        NativeToml,
        SentinelCoreConfig,
    },
    Endpoints,
    SentinelError,
};

#[derive(Debug, Clone, Deserialize)]
struct SentinelConfigToml {
    log: LogToml,
    host: HostToml,
    ipfs: IpfsConfig,
    native: NativeToml,
    batching: BatchingToml,
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
    batching: BatchingConfig,
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
            batching: BatchingConfig::from_toml(&toml.batching)?,
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

    pub fn is_validating(&self, side: &BridgeSide) -> bool {
        if side.is_native() {
            self.native.is_validating()
        } else {
            self.host.is_validating()
        }
    }

    pub fn pnetwork_hub(&self, side: &BridgeSide) -> EthAddress {
        if side.is_native() {
            *self.native.pnetwork_hub()
        } else {
            *self.host.pnetwork_hub()
        }
    }

    pub fn chain_id(&self, side: &BridgeSide) -> EthChainId {
        if side.is_native() {
            self.native.chain_id()
        } else {
            self.host.chain_id()
        }
    }

    pub fn gas_price(&self, side: &BridgeSide) -> Option<u64> {
        if side.is_native() {
            *self.native.gas_price()
        } else {
            *self.host.gas_price()
        }
    }

    pub fn gas_limit(&self, side: &BridgeSide) -> usize {
        if side.is_native() {
            *self.native.gas_limit()
        } else {
            *self.host.gas_limit()
        }
    }

    pub fn mcids(&self) -> Vec<MetadataChainId> {
        vec![self.native().mcid(), self.host.mcid()]
    }

    pub fn governance_address(&self, mcid: &MetadataChainId) -> Option<EthAddress> {
        // NOTE: The governance contract lives on one chain only
        if mcid == self.governance().mcid() {
            Some(*self.governance().governance_address())
        } else {
            None
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
