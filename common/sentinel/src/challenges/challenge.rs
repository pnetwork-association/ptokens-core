use common::MIN_DATA_SENSITIVITY_LEVEL;
use common_eth::{EthPrivateKey, EthSignature, EthSigningCapabilities};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

use super::{ChallengePendingEvent, ChallengesError};
use crate::{Actor, DbKey, DbUtilsT, SentinelError};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Getters, Constructor)]
pub struct Challenge {
    nonce: U256,
    actor: Actor,
    timestamp: u64,
    mcid: MetadataChainId,
    challenger_address: EthAddress,
}

impl Challenge {
    #[cfg(test)]
    pub(crate) fn random() -> Self {
        use rand::Rng;
        Self::new(
            U256::from(rand::thread_rng().gen_range(0..100_000_000)),
            Actor::random(),
            rand::thread_rng().gen_range(0..100_000_000),
            MetadataChainId::default(),
            EthAddress::random(),
        )
    }

    pub fn sign(&self, pk: &EthPrivateKey) -> Result<EthSignature, ChallengesError> {
        let bs = self.abi_encode()?;
        Ok(pk.hash_and_sign_msg_with_eth_prefix(&bs)?)
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

impl DbUtilsT for Challenge {
    fn key(&self) -> Result<DbKey, SentinelError> {
        Ok(self.hash()?.into())
    }

    fn sensitivity() -> Option<u8> {
        MIN_DATA_SENSITIVITY_LEVEL
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, SentinelError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
