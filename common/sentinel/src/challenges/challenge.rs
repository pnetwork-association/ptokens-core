use common_metadata::MetadataChainId;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::{Address as EthAddress, U256};

use super::ChallengePendingEvent;
use crate::Actor;

/* Reference:
From: https://github.com/pnetwork-association/pnetwork/blob/14d11b116da6abf70cba11e0fd931686f77f22b5/packages/ptokens-evm-contracts/contracts/interfaces/IPNetworkHub.sol#L47C1-L54C6
     struct Challenge {
        uint256 nonce;
        address actor;
        address challenger;
        ActorTypes actorType;
        uint64 timestamp;
        bytes4 networkId;
    }
*/

#[derive(Debug, Clone, Eq, PartialEq, Getters, Constructor)]
pub struct Challenge {
    nonce: U256,
    actor: Actor,
    timestamp: u64,
    mcid: MetadataChainId,
    challenger_address: EthAddress,
}

impl From<&ChallengePendingEvent> for Challenge {
    fn from(event: &ChallengePendingEvent) -> Self {
        Self {
            mcid: *event.mcid(),
            nonce: *event.nonce(),
            timestamp: *event.timestamp(),
            challenger_address: *event.challenger_address(),
            actor: Actor::new(*event.actor_type(), *event.actor_address()),
        }
    }
}
