use std::fmt;

use common::crypto_utils::keccak_hash_bytes;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use super::{type_aliases::Hash, ActorType};

#[derive(Clone, Debug, Eq, PartialEq, Constructor, Getters, Serialize, Deserialize)]
pub struct Actor {
    actor_type: ActorType,
    actor_address: EthAddress,
}

impl Actor {
    pub(super) fn to_leaf(&self) -> Hash {
        keccak_hash_bytes(&[self.actor_address.as_bytes(), self.actor_type.as_bytes()].concat()).into()
    }
}

impl From<EthAddress> for Actor {
    fn from(a: EthAddress) -> Self {
        Self::from(&a)
    }
}

impl From<&EthAddress> for Actor {
    fn from(a: &EthAddress) -> Self {
        // NOTE: This is the sentinel code base, hence we can assume the actor type
        Actor::new(ActorType::Sentinel, *a)
    }
}

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "{e}"),
        }
    }
}
