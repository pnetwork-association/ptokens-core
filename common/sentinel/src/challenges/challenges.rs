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
    pub fn from_sub_mat(
        sub_mat: &EthSubmissionMaterial,
        pnetwork_hub: &EthAddress,
        sentinel_address: &EthAddress,
    ) -> Result<Self, ChallengesError> {
        debug!("parsing challenges from sub mat...");
        ChallengePendingEvents::from_sub_mat(sub_mat, pnetwork_hub)
            .map(|events| {
                events
                    .iter()
                    .map(Challenge::from)
                    .filter(|c| c.actor().actor_address() == sentinel_address)
                    .collect()
            })
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

    use ethereum_types::{H256 as EthHash, U256};

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
        let sentinel_address = EthAddress::from_str("0x73659A0f105905121EDbF44Fb476B97c785688EC").unwrap();
        let pnetwork_hub = EthAddress::from_str("0x6153ec976A5B3886caF3A88D8d994c4CEC24203E").unwrap();
        let events = Challenges::from_sub_mat(&sub_mat, &pnetwork_hub, &sentinel_address).unwrap();
        assert_eq!(events.len(), 1);
        let expected_challenge = get_expected_challenge();
        assert_eq!(events[0], expected_challenge);
    }

    #[test]
    fn should_get_challenges_from_sub_mat_2() {
        let sub_mat = get_sample_sub_mat_with_challenge_pending_event_2();
        let pnetwork_hub = EthAddress::from_str("0xf28910cc8f21e9314ed50627c11de36bc0b7338f").unwrap();
        let sentinel_address = EthAddress::from_str("0x73659A0f105905121EDbF44Fb476B97c785688EC").unwrap();
        let challenges = Challenges::from_sub_mat(&sub_mat, &pnetwork_hub, &sentinel_address).unwrap();
        assert_eq!(challenges.len(), 1);
        let challenge = challenges[0].clone();
        let expected_challenge = Challenge::new(
            U256::from(10),
            Actor::new(
                ActorType::Sentinel,
                EthAddress::from_str("0x73659a0f105905121edbf44fb476b97c785688ec").unwrap(),
            ),
            1698667343,
            NetworkId::try_from("polygon").unwrap(),
            EthAddress::from_str("0xe5de26b691d615353a03285405b6ee08c7974926").unwrap(),
        );
        assert_eq!(challenge, expected_challenge);
        let expected_encoded = "000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000073659a0f105905121edbf44fb476b97c785688ec000000000000000000000000e5de26b691d615353a03285405b6ee08c7974926000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000653f9b4ff9b459a100000000000000000000000000000000000000000000000000000000";
        let encoded = hex::encode(challenge.abi_encode().unwrap());
        assert_eq!(encoded, expected_encoded);
        let expected_id =
            EthHash::from_str("0008a5f5033a80e59882d11c1c65a6453e74aad9d6a32e63379ddaeb386872f2").unwrap();
        let id = challenge.id().unwrap();
        assert_eq!(id, expected_id);
    }
}
