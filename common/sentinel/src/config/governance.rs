use std::{result::Result, str::FromStr};

use derive_getters::Getters;
use ethereum_types::Address as EthAddress;
use serde::Deserialize;

use crate::{NetworkId, SentinelError};

#[derive(Debug, Clone, Deserialize, Getters)]
pub struct GovernanceToml {
    address: String,
    network_id: String,
}

#[derive(Debug, Clone, Default, Getters)]
pub struct GovernanceConfig {
    network_id: NetworkId,
    governance_address: EthAddress,
}

impl TryFrom<&GovernanceToml> for GovernanceConfig {
    type Error = SentinelError;

    fn try_from(toml: &GovernanceToml) -> Result<Self, Self::Error> {
        Ok(Self {
            network_id: NetworkId::try_from(toml.network_id())?,
            governance_address: EthAddress::from_str(toml.address())?,
        })
    }
}
