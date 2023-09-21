use std::{fmt, str::FromStr};

use derive_more::{Constructor, Deref};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use crate::SentinelError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct UserOpUniqueId(EthHash);

impl FromStr for UserOpUniqueId {
    type Err = SentinelError;

    fn from_str(s: &str) -> Result<Self, SentinelError> {
        Ok(Self(EthHash::from_str(s)?))
    }
}

impl From<EthHash> for UserOpUniqueId {
    fn from(h: EthHash) -> UserOpUniqueId {
        UserOpUniqueId::new(h)
    }
}

impl fmt::Display for UserOpUniqueId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = hex::encode(self.0);
        write!(f, "0x{s}")
    }
}
