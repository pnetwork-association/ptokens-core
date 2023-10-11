use std::fmt;

use common_eth::{EthReceipts, EthSubmissionMaterial};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use super::{type_aliases::Hash, Actor, ActorsError, ActorsPropagatedEvent, ACTORS_PROPAGATED_EVENT_TOPIC};

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor, Getters, Serialize, Deserialize)]
pub struct Actors {
    epoch: U256,
    tx_hash: EthHash,
    actors: Vec<Actor>,
    mcid: MetadataChainId,
    governance_contract: EthAddress,
}

impl Actors {
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.actors.len()
    }

    #[cfg(test)]
    pub fn default_with_actors(actors: Vec<Actor>) -> Self {
        Self {
            actors,
            ..Default::default()
        }
    }

    pub(super) fn to_leaves(&self) -> Vec<Hash> {
        self.actors().iter().map(Actor::to_leaf).collect()
    }

    pub fn from_sub_mat(
        sub_mat: &EthSubmissionMaterial,
        governance_contract: EthAddress,
        mcid: MetadataChainId,
    ) -> Result<Option<Self>, ActorsError> {
        let block_hash = sub_mat.hash.unwrap_or_default();
        let governance_receipts = EthReceipts::new(sub_mat.get_receipts())
            .get_receipts_containing_logs_from_address_and_with_topic(
                &governance_contract,
                &ACTORS_PROPAGATED_EVENT_TOPIC,
            );
        let logs =
            governance_receipts.get_logs_from_address_with_topic(&governance_contract, &ACTORS_PROPAGATED_EVENT_TOPIC);

        if logs.is_empty() {
            return Ok(None);
        };

        if logs.len() > 1 {
            return Err(ActorsError::TooManyLogs {
                chain: mcid,
                block_hash: format!("0x{}", hex::encode(block_hash.as_bytes())),
            });
        };

        let tx_hash = governance_receipts[0].transaction_hash;
        let event = ActorsPropagatedEvent::try_from(&logs[0])?;
        let actors = event
            .actor_types()
            .iter()
            .zip(event.actor_addresses().iter())
            .map(|(actor_type, actor_address)| Actor::new(actor_type.clone(), *actor_address))
            .collect::<Vec<Actor>>();

        Ok(Some(Self::new(
            *event.epoch(),
            tx_hash,
            actors,
            mcid,
            governance_contract,
        )))
    }
}

impl fmt::Display for Actors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "{e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::actors::test_utils::{get_sample_actors, get_sample_actors_propagated_sub_mat};

    fn get_governance_address() -> EthAddress {
        EthAddress::from_str("0x186d7656ca8e16d6E04B2a87b196d473f3566F54").unwrap()
    }
    #[test]
    fn should_get_actors_from_sub_mat() {
        let sub_mat = get_sample_actors_propagated_sub_mat();
        let governance_contract = get_governance_address();
        let mcid = MetadataChainId::PolygonMainnet;
        let actors = Actors::from_sub_mat(&sub_mat, governance_contract, mcid).unwrap();
        let expected_actors = get_sample_actors();
        assert_eq!(actors, Some(expected_actors))
    }

    #[test]
    fn should_not_get_actors_from_sub_mat_if_not_extant() {
        let sub_mat = get_sample_actors_propagated_sub_mat();
        let governance_contract = EthAddress::random();
        assert_ne!(governance_contract, get_governance_address());
        let mcid = MetadataChainId::PolygonMainnet;
        let actors = Actors::from_sub_mat(&sub_mat, governance_contract, mcid).unwrap();
        assert_eq!(actors, None);
    }
}
