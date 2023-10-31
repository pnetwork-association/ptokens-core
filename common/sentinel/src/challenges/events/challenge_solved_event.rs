use std::str::FromStr;

use common_eth::EthSubmissionMaterial;
use ethereum_types::{Address as EthAddress, H256 as EthHash};

use crate::challenges::{Challenge, ChallengeEvent, Challenges, ChallengesError};

lazy_static! {
    pub static ref CHALLENGE_SOLVED_EVENT_TOPIC: EthHash =
        EthHash::from_str("ea8e312ba84107c42fec02bd07ae90c5e0947f25b0cffaa43f35e0faf27eec48")
            .expect("this not to fail");
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChallengeSolvedEvents(Vec<ChallengeEvent>);

impl ChallengeSolvedEvents {
    pub(crate) fn from_sub_mat(
        sub_mat: &EthSubmissionMaterial,
        pnetwork_hub: &EthAddress,
        _sentinel_address: &EthAddress,
    ) -> Result<Self, ChallengesError> {
        let logs = sub_mat
            .receipts
            .get_logs_from_address_with_topic(pnetwork_hub, &CHALLENGE_SOLVED_EVENT_TOPIC);
        let events = logs
            .iter()
            .map(ChallengeEvent::try_from)
            .filter(|_| true)
            .collect::<Result<Vec<ChallengeEvent>, ChallengesError>>()?;
        Ok(Self(events))
    }

    fn to_challenges(&self) -> Challenges {
        Challenges::new(self.0.iter().map(Challenge::from).collect())
    }

    pub(crate) fn to_ids(&self) -> Result<Vec<EthHash>, ChallengesError> {
        self.to_challenges().iter().map(|c| c.id()).collect()
    }
}
