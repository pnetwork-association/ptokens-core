use common::crypto_utils::keccak_hash_bytes;
use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;

use super::{type_aliases::Hash, ActorType};

#[derive(Clone, Debug, Deref, Constructor)]
pub struct Actors(Vec<Actor>);

impl Actors {
    pub(super) fn to_leaves(&self) -> Vec<Hash> {
        self.iter().map(Actor::to_leaf).collect()
    }
}

#[derive(Clone, Debug, Constructor, Getters)]
pub struct Actor {
    actor_type: ActorType,
    actor_address: EthAddress,
}

impl Actor {
    pub(super) fn to_leaf(&self) -> Hash {
        keccak_hash_bytes(&[self.actor_address.as_bytes(), self.actor_type.as_bytes()].concat()).into()
    }
}
