use common_eth::EthSubmissionMaterial;
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use super::{Challenge, ChallengePendingEvents, ChallengesError};

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, Serialize, Deserialize)]
pub struct Challenges(Vec<Challenge>);

impl Challenges {
    pub fn from_sub_mat(sub_mat: &EthSubmissionMaterial, pnetwork_hub: &EthAddress) -> Result<Self, ChallengesError> {
        ChallengePendingEvents::from_sub_mat(sub_mat, pnetwork_hub)
            .map(|events| events.iter().map(Challenge::from).collect())
            .map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common_metadata::MetadataChainId;
    use ethereum_types::U256;

    use super::*;
    use crate::{challenges::test_utils::get_sample_sub_mat_with_challenge_pending_event, Actor, ActorType};

    fn get_expected_challenge() -> Challenge {
        Challenge::new(
            U256::from(0),
            Actor::new(
                ActorType::Sentinel,
                EthAddress::from_str("0x73659a0f105905121edbf44fb476b97c785688ec").unwrap(),
            ),
            1697147101,
            MetadataChainId::PolygonMainnet,
            EthAddress::from_str("0xada2de876567a06ed79b0b29ae6ab2e142129e51").unwrap(),
        )
    }

    #[test]
    fn should_get_challenges_from_sub_mat() {
        let sub_mat = get_sample_sub_mat_with_challenge_pending_event();
        let pnetwork_hub = EthAddress::from_str("0x6153ec976A5B3886caF3A88D8d994c4CEC24203E").unwrap();
        let events = Challenges::from_sub_mat(&sub_mat, &pnetwork_hub).unwrap();
        assert_eq!(events.len(), 1);
        let expected_event = get_expected_challenge();
        assert_eq!(events[0], expected_event);
    }
}
