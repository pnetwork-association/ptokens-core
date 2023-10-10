use std::str::FromStr;

use common_eth::{EthLog, EthLogExt};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use super::ActorsError;

lazy_static! {
    pub static ref ACTORS_PROPAGATED_EVENT_TOPIC: EthHash =
        EthHash::from_str("7d394dea630b3e42246f284e4e4b75cff4f959869b3d753639ba8ae6120c67c3")
            .expect("this not to fail");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ActorType {
    Governance = 0,
    Guardian   = 1,
    Sentinel   = 2,
}

impl TryFrom<&U256> for ActorType {
    type Error = ActorsError;

    fn try_from(u: &U256) -> Result<Self, Self::Error> {
        match u.as_u64() {
            0 => Ok(Self::Governance),
            1 => Ok(Self::Guardian),
            2 => Ok(Self::Sentinel),
            n => Err(ActorsError::CannotGetActorType(n)),
        }
    }
}

impl Default for ActorType {
    fn default() -> Self {
        Self::Governance
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ActorsPropagatedEvent {
    epoch: U256,
    actor_types: Vec<ActorType>,
    actor_addresses: Vec<EthAddress>,
}

impl TryFrom<EthLog> for ActorsPropagatedEvent {
    type Error = ActorsError;

    fn try_from(log: EthLog) -> Result<Self, Self::Error> {
        let expected_num_topics = 2;

        if log.topics.is_empty() {
            return Err(ActorsError::WrongNumberOfTopics {
                got: 0,
                expected: expected_num_topics,
            });
        };

        if log.topics[0] != *ACTORS_PROPAGATED_EVENT_TOPIC {
            return Err(ActorsError::WrongTopic { topic: log.topics[0] });
        };

        if log.topics.len() != expected_num_topics {
            return Err(ActorsError::WrongNumberOfTopics {
                got: log.topics.len(),
                expected: expected_num_topics,
            });
        };

        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::Array(Box::new(EthAbiParamType::Address)),
                EthAbiParamType::Array(Box::new(EthAbiParamType::Uint(8))),
            ],
            &log.get_data(),
        )?;

        let actor_addresses = match tokens[0] {
            EthAbiToken::Array(ref vec_of_tokens) => {
                let mut addresses = vec![];
                for token in vec_of_tokens {
                    let address = match token {
                        EthAbiToken::Address(a) => Ok(a),
                        token => Err(ActorsError::WrongToken {
                            got: token.clone(),
                            expected: "Address".to_string(),
                        }),
                    }?;
                    addresses.push(*address);
                }
                Ok(addresses)
            },
            ref token => Err(ActorsError::WrongToken {
                got: token.clone(),
                expected: "Array".to_string(),
            }),
        }?;

        let epoch = U256::from_big_endian(log.topics[1].as_bytes());

        let actor_types = match tokens[1] {
            EthAbiToken::Array(ref vec_of_tokens) => {
                let mut actor_types = vec![];
                for token in vec_of_tokens {
                    let actor_type = match token {
                        EthAbiToken::Uint(n) => ActorType::try_from(n),
                        token => Err(ActorsError::WrongToken {
                            got: token.clone(),
                            expected: "Uint".to_string(),
                        }),
                    }?;
                    actor_types.push(actor_type);
                }
                Ok(actor_types)
            },
            ref token => Err(ActorsError::WrongToken {
                got: token.clone(),
                expected: "Array".to_string(),
            }),
        }?;

        let num_types = actor_types.len();
        let num_actors = actor_addresses.len();
        if num_actors != num_types {
            return Err(ActorsError::ActorAddressesAndTypesMismatch { num_actors, num_types });
        };

        Ok(Self {
            epoch,
            actor_addresses,
            actor_types,
        })
    }
}
