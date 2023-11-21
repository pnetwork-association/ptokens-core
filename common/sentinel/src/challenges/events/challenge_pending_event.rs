use std::str::FromStr;

use common_eth::EthSubmissionMaterial;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash};

use crate::challenges::{ChallengeEvent, ChallengesError};

lazy_static! {
    pub(in crate::challenges) static ref CHALLENGE_PENDING_EVENT_TOPIC: EthHash =
        EthHash::from_str("6fd10f30cfab9f88f3ab98604755507ed88159095ce2102e483f2b4f441d6f14")
            .expect("this not to fail");
}

#[derive(Debug, Clone, Eq, PartialEq, Deref, Constructor)]
pub(in crate::challenges) struct ChallengePendingEvents(Vec<ChallengeEvent>);

impl ChallengePendingEvents {
    pub(crate) fn from_sub_mat(
        sub_mat: &EthSubmissionMaterial,
        pnetwork_hub: &EthAddress,
    ) -> Result<Self, ChallengesError> {
        let logs = sub_mat
            .receipts
            .get_logs_from_address_with_topic(pnetwork_hub, &CHALLENGE_PENDING_EVENT_TOPIC);
        let events = logs
            .iter()
            .map(ChallengeEvent::try_from)
            .collect::<Result<Vec<ChallengeEvent>, ChallengesError>>()?;
        Ok(Self::new(events))
    }
}

#[cfg(test)]
mod tests {
    use ethereum_types::U256;

    use super::*;
    use crate::{
        challenges::test_utils::{
            get_sample_log_with_challenge_pending_event,
            get_sample_sub_mat_with_challenge_pending_event,
        },
        ActorType,
        NetworkId,
    };

    fn get_expected_challenge_event() -> ChallengeEvent {
        ChallengeEvent {
            nonce: U256::from(0),
            timestamp: 1697147101,
            actor_type: ActorType::Sentinel,
            network_id: NetworkId::try_from("polygon").unwrap(),
            actor_address: EthAddress::from_str("0x73659a0f105905121edbf44fb476b97c785688ec").unwrap(),
            challenger_address: EthAddress::from_str("0xada2de876567a06ed79b0b29ae6ab2e142129e51").unwrap(),
        }
    }

    #[test]
    fn should_get_challenge_pending_event_from_eth_log() {
        let log = get_sample_log_with_challenge_pending_event();
        let event = ChallengeEvent::try_from(log).unwrap();
        let expected_event = get_expected_challenge_event();
        assert_eq!(event, expected_event);
    }

    #[test]
    fn should_get_challenge_pending_events_from_sub_mat() {
        let sub_mat = get_sample_sub_mat_with_challenge_pending_event();
        let pnetwork_hub = EthAddress::from_str("0x6153ec976A5B3886caF3A88D8d994c4CEC24203E").unwrap();
        let events = ChallengePendingEvents::from_sub_mat(&sub_mat, &pnetwork_hub).unwrap();
        assert_eq!(events.len(), 1);
        let expected_event = get_expected_challenge_event();
        assert_eq!(events[0], expected_event);
    }
}
