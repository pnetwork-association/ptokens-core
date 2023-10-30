use common_eth::EthSubmissionMaterial;
use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::{Challenge, ChallengePendingEvents, ChallengesError};
use crate::WebSocketMessagesEncodable;

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deref, Serialize, Deserialize)]
pub struct Challenges(Vec<Challenge>);

impl Challenges {
    pub fn from_sub_mat(sub_mat: &EthSubmissionMaterial, pnetwork_hub: &EthAddress) -> Result<Self, ChallengesError> {
        ChallengePendingEvents::from_sub_mat(sub_mat, pnetwork_hub)
            .map(|events| events.iter().map(Challenge::from).collect())
            .map(Self::new)
    }
}

impl TryFrom<WebSocketMessagesEncodable> for Challenges {
    type Error = ChallengesError;

    fn try_from(m: WebSocketMessagesEncodable) -> Result<Self, Self::Error> {
        let j = Json::try_from(m)?;
        Ok(serde_json::from_value(j)?)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethereum_types::U256;

    use super::*;
    use crate::{
        challenges::test_utils::{
            get_sample_sub_mat_with_challenge_pending_event,
            get_sample_sub_mat_with_challenge_pending_event_2,
        },
        Actor,
        ActorType,
        NetworkId,
    };

    fn get_expected_challenge() -> Challenge {
        Challenge::new(
            U256::from(0),
            Actor::new(
                ActorType::Sentinel,
                EthAddress::from_str("0x73659a0f105905121edbf44fb476b97c785688ec").unwrap(),
            ),
            1697147101,
            NetworkId::try_from("polygon").unwrap(),
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

    #[test]
    fn should_get_challenges_from_sub_mat_2() {
        let sub_mat = get_sample_sub_mat_with_challenge_pending_event_2();
        let pnetwork_hub = EthAddress::from_str("0xf28910cc8f21e9314ed50627c11de36bc0b7338f").unwrap();
        let events = Challenges::from_sub_mat(&sub_mat, &pnetwork_hub).unwrap();
        assert_eq!(events.len(), 1);
        let expected_event = Challenge::new(
            U256::from(10),
            Actor::new(
                ActorType::Sentinel,
                EthAddress::from_str("0x73659a0f105905121edbf44fb476b97c785688ec").unwrap(),
            ),
            1698667343,
            NetworkId::try_from("polygon").unwrap(),
            EthAddress::from_str("0xe5de26b691d615353a03285405b6ee08c7974926").unwrap(),
        );
        assert_eq!(events[0], expected_event);
    }
}
