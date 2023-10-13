use std::str::FromStr;

use common_eth::{EthLog, EthLogExt, EthSubmissionMaterial};
use common_metadata::MetadataChainId;
use derive_getters::Getters;
use derive_more::{Constructor, Deref};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use super::ChallengesError;
use crate::{Actor, ActorType, NetworkId};

lazy_static! {
    pub(super) static ref CHALLENGE_PENDING_EVENT_TOPIC: EthHash =
        EthHash::from_str("6fd10f30cfab9f88f3ab98604755507ed88159095ce2102e483f2b4f441d6f14")
            .expect("this not to fail");
}

#[derive(Debug, Clone, Eq, PartialEq, Deref, Constructor)]
pub struct ChallengePendingEvents(Vec<ChallengePendingEvent>);

impl ChallengePendingEvents {
    pub fn from_sub_mat(sub_mat: &EthSubmissionMaterial, pnetwork_hub: &EthAddress) -> Result<Self, ChallengesError> {
        let logs = sub_mat
            .receipts
            .get_logs_from_address_with_topic(pnetwork_hub, &CHALLENGE_PENDING_EVENT_TOPIC);
        let events = logs
            .iter()
            .map(ChallengePendingEvent::try_from)
            .collect::<Result<Vec<ChallengePendingEvent>, ChallengesError>>()?;
        Ok(Self::new(events))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Getters, Constructor)]
pub struct ChallengePendingEvent {
    nonce: U256,
    timestamp: u64,
    mcid: MetadataChainId,
    actor_type: ActorType,
    actor_address: EthAddress,
    challenger_address: EthAddress,
}

impl TryFrom<EthLog> for ChallengePendingEvent {
    type Error = ChallengesError;

    fn try_from(log: EthLog) -> Result<Self, Self::Error> {
        Self::try_from(&log)
    }
}

impl TryFrom<&EthLog> for ChallengePendingEvent {
    type Error = ChallengesError;

    fn try_from(log: &EthLog) -> Result<Self, Self::Error> {
        if log.topics.is_empty() {
            return Err(Self::Error::NoTopics);
        }

        if log.topics[0] != *CHALLENGE_PENDING_EVENT_TOPIC {
            return Err(Self::Error::WrongTopic);
        }

        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::Uint(256),
                EthAbiParamType::Address,
                EthAbiParamType::Address,
                EthAbiParamType::Uint(8),
                EthAbiParamType::Uint(64),
                EthAbiParamType::FixedBytes(4),
            ],
            &log.get_data(),
        )?;

        Self::try_from(tokens)
    }
}

impl TryFrom<Vec<EthAbiToken>> for ChallengePendingEvent {
    type Error = ChallengesError;

    fn try_from(tokens: Vec<EthAbiToken>) -> Result<Self, Self::Error> {
        const CHALLENGE_NUM_TOKENS: usize = 6;

        if tokens.len() != CHALLENGE_NUM_TOKENS {
            return Err(Self::Error::IncorrectNumTokens {
                got: tokens.len(),
                expected: CHALLENGE_NUM_TOKENS,
            });
        };

        let nonce = match tokens[0] {
            EthAbiToken::Uint(n) => Ok(n),
            ref token => Err(Self::Error::WrongToken {
                got: token.clone(),
                expected: "Uint".to_string(),
            }),
        }?;

        let actor_address = match tokens[1] {
            EthAbiToken::Address(a) => Ok(a),
            ref token => Err(Self::Error::WrongToken {
                got: token.clone(),
                expected: "Address".to_string(),
            }),
        }?;

        let challenger_address = match tokens[2] {
            EthAbiToken::Address(a) => Ok(a),
            ref token => Err(Self::Error::WrongToken {
                got: token.clone(),
                expected: "Address".to_string(),
            }),
        }?;

        let actor_type = match tokens[3] {
            EthAbiToken::Uint(ref n) => Ok(ActorType::try_from(n)?),
            ref token => Err(Self::Error::WrongToken {
                got: token.clone(),
                expected: "Uint".to_string(),
            }),
        }?;

        let timestamp = match tokens[4] {
            EthAbiToken::Uint(n) => Ok(n.as_u64()),
            ref token => Err(Self::Error::WrongToken {
                got: token.clone(),
                expected: "Uint".to_string(),
            }),
        }?;

        let mcid = match tokens[5] {
            EthAbiToken::FixedBytes(ref bs) => Ok(MetadataChainId::try_from(NetworkId::try_from(bs)?)?),
            ref token => Err(Self::Error::WrongToken {
                got: token.clone(),
                expected: "FixedBytes".to_string(),
            }),
        }?;

        Ok(Self::new(
            nonce,
            timestamp,
            mcid,
            actor_type,
            actor_address,
            challenger_address,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenges::test_utils::{
        get_sample_log_with_challenge_pending_event,
        get_sample_sub_mat_with_challenge_pending_event,
    };

    fn get_expected_challenge_pending_event() -> ChallengePendingEvent {
        ChallengePendingEvent {
            nonce: U256::from(0),
            timestamp: 1697147101,
            mcid: MetadataChainId::PolygonMainnet,
            actor_type: ActorType::Sentinel,
            actor_address: EthAddress::from_str("0x73659a0f105905121edbf44fb476b97c785688ec").unwrap(),
            challenger_address: EthAddress::from_str("0xada2de876567a06ed79b0b29ae6ab2e142129e51").unwrap(),
        }
    }

    #[test]
    fn should_get_challenge_pending_event_from_eth_log() {
        let log = get_sample_log_with_challenge_pending_event();
        let event = ChallengePendingEvent::try_from(log).unwrap();
        let expected_event = get_expected_challenge_pending_event();
        assert_eq!(event, expected_event);
    }

    #[test]
    fn should_get_challenge_pending_events_from_sub_mat() {
        let sub_mat = get_sample_sub_mat_with_challenge_pending_event();
        let pnetwork_hub = EthAddress::from_str("0x6153ec976A5B3886caF3A88D8d994c4CEC24203E").unwrap();
        let events = ChallengePendingEvents::from_sub_mat(&sub_mat, &pnetwork_hub).unwrap();
        assert_eq!(events.len(), 1);
        let expected_event = get_expected_challenge_pending_event();
        assert_eq!(events[0], expected_event);
    }
}
