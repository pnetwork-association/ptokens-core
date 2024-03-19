use common::MIN_DATA_SENSITIVITY_LEVEL;
use common_eth::{EthPrivateKey, EthSignature, EthSigningCapabilities};
use common_network_ids::NetworkId;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::{ChallengeEvent, ChallengeResponseSignatureInfo, ChallengesError};
use crate::{Actor, ActorInclusionProof, DbKey, DbUtilsT, SentinelError, WebSocketMessagesEncodable};

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

// FIXME Do we want/need to track the `ChallengeState` in here?

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Getters, Constructor)]
pub struct Challenge {
    nonce: U256,
    actor: Actor,
    timestamp: u64,
    network_id: NetworkId,
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
            NetworkId::default(),
            EthAddress::random(),
        )
    }

    pub fn sign(&self, pk: &EthPrivateKey) -> Result<EthSignature, ChallengesError> {
        let id = self.id()?;
        Ok(pk.hash_and_sign_msg_with_eth_prefix(id.as_bytes())?)
    }

    pub fn get_response_sig_info(
        &self,
        proof: ActorInclusionProof,
        signing_key: &EthPrivateKey,
    ) -> Result<ChallengeResponseSignatureInfo, ChallengesError> {
        debug!("getting challenge response signature info...");
        let id = self.id()?;
        let sig = self.sign(signing_key)?;
        let signer = signing_key.to_address();
        debug!("    id: {id}");
        debug!("   sig: {sig}");
        debug!("signer: {signer}");
        Ok(ChallengeResponseSignatureInfo::new(id, signer, proof, sig.into()))
    }
}

impl From<&ChallengeEvent> for Challenge {
    fn from(event: &ChallengeEvent) -> Self {
        Self {
            nonce: *event.nonce(),
            timestamp: *event.timestamp(),
            network_id: *event.network_id(),
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

impl TryFrom<WebSocketMessagesEncodable> for Challenge {
    type Error = ChallengesError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        let j = Json::try_from(m)?;
        Ok(serde_json::from_value(j)?)
    }
}
