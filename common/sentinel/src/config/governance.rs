use std::{result::Result, str::FromStr};

use common_metadata::MetadataChainId;
use derive_getters::Getters;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use crate::SentinelError;

#[derive(Debug, Clone, Deserialize, Getters)]
pub struct GovernanceToml {
    mcid: String,
    address: String,
}

#[derive(Debug, Clone, Default, Getters)]
pub struct GovernanceConfig {
    mcid: MetadataChainId,
    governance_address: EthAddress,
}

impl TryFrom<&GovernanceToml> for GovernanceConfig {
    type Error = SentinelError;

    fn try_from(toml: &GovernanceToml) -> Result<Self, Self::Error> {
        Ok(Self {
            mcid: MetadataChainId::from_str(toml.mcid())?,
            governance_address: EthAddress::from_str(toml.address())?,
        })
    }
}
