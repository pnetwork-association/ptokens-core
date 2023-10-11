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
