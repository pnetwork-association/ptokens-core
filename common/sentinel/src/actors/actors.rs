use derive_more::Deref;
use ethereum_types::Address as EthAddress;

use super::ActorType;

#[derive(Clone, Debug, Deref)]
pub struct Actors(Vec<Actor>);

#[derive(Clone, Debug)]
pub struct Actor {
    actor_type: ActorType,
    actor_address: EthAddress,
}
