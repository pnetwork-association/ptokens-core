use common_eth::{EthPrivateKey, EthSignature};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

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

// FIXME Do we want/need to track the `ChallengeStatus` in here?

#[derive(Debug, Clone, Eq, PartialEq, Getters, Constructor)]
pub struct Challenge {
    nonce: U256,
    actor: Actor,
    timestamp: u64,
    mcid: MetadataChainId,
    challenger_address: EthAddress,
}

impl Challenge {
    fn sign(&self, pk: &EthPrivateKey) -> EthSignature {
        todo!("sign a challenge");
    }

    fn abi_encode(&self) -> Vec<u8> {
        todo!("abi_encode challenge");
    }

    pub(super) fn hash(&self) -> EthHash {
        let encoded = self.abi_encode();
        todo!("abi encode and hash this")
    }
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
