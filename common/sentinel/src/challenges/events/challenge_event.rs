use common_eth::{EthLog, EthLogExt};
use derive_getters::Getters;
use derive_more::Constructor;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, U256};

use crate::{
    challenges::{
        events::{CHALLENGE_PENDING_EVENT_TOPIC, CHALLENGE_SOLVED_EVENT_TOPIC},
        ChallengesError,
    },
    ActorType,
    NetworkId,
};

#[derive(Debug, Clone, Eq, PartialEq, Getters, Constructor)]
pub(in crate::challenges) struct ChallengeEvent {
    pub(super) nonce: U256,
    pub(super) timestamp: u64,
    pub(super) network_id: NetworkId,
    pub(super) actor_type: ActorType,
    pub(super) actor_address: EthAddress,
    pub(super) challenger_address: EthAddress,
}

impl TryFrom<EthLog> for ChallengeEvent {
    type Error = ChallengesError;

    fn try_from(log: EthLog) -> Result<Self, Self::Error> {
        Self::try_from(&log)
    }
}

impl TryFrom<&EthLog> for ChallengeEvent {
    type Error = ChallengesError;

    fn try_from(log: &EthLog) -> Result<Self, Self::Error> {
        if log.topics.is_empty() {
            return Err(Self::Error::NoTopics);
        }

        let allowed_topics = [*CHALLENGE_PENDING_EVENT_TOPIC, *CHALLENGE_SOLVED_EVENT_TOPIC];

        if !allowed_topics.contains(&log.topics[0]) {
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

impl TryFrom<Vec<EthAbiToken>> for ChallengeEvent {
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

        let network_id = match tokens[5] {
            EthAbiToken::FixedBytes(ref bs) => Ok(NetworkId::try_from(bs)?),
            ref token => Err(Self::Error::WrongToken {
                got: token.clone(),
                expected: "FixedBytes".to_string(),
            }),
        }?;

        Ok(Self::new(
            nonce,
            timestamp,
            network_id,
            actor_type,
            actor_address,
            challenger_address,
        ))
    }
}
