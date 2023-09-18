use std::str::FromStr;

use derive_more::Deref;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use crate::SentinelError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Deref)]
pub struct UserOpUniqueId(EthHash);

impl FromStr for UserOpUniqueId {
    type Err = SentinelError;

    fn from_str(s: &str) -> Result<Self, SentinelError> {
        Ok(Self(EthHash::from_str(s)?))
    }
}
