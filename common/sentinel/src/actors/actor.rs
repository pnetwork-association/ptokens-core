use std::fmt;

use common::crypto_utils::keccak_hash_bytes;
use common_eth::EthLog;
use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

use super::{type_aliases::Hash, ActorType, ActorsError};
use crate::user_ops::CANCELLED_USER_OP_TOPIC;

#[derive(Clone, Default, Copy, Debug, Eq, PartialEq, Constructor, Getters, Serialize, Deserialize)]
pub struct Actor {
    actor_type: ActorType,
    actor_address: EthAddress,
}

impl Actor {
    #[cfg(test)]
    pub(crate) fn random() -> Self {
        Self::new(ActorType::random(), EthAddress::random())
    }

    pub(super) fn to_leaf(self) -> Hash {
        keccak_hash_bytes(&[self.actor_address.as_bytes(), self.actor_type.as_bytes()].concat()).into()
    }
}

impl From<EthAddress> for Actor {
    fn from(a: EthAddress) -> Self {
        Self::from(&a)
    }
}

impl From<&EthAddress> for Actor {
    fn from(a: &EthAddress) -> Self {
        // NOTE: This is the sentinel code base, hence we can assume the actor type
        Actor::new(ActorType::Sentinel, *a)
    }
}

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "{e}"),
        }
    }
}

impl TryFrom<EthLog> for Actor {
    type Error = ActorsError;

    fn try_from(l: EthLog) -> Result<Self, Self::Error> {
        Self::try_from(&l)
    }
}

impl TryFrom<&EthLog> for Actor {
    type Error = ActorsError;

    fn try_from(l: &EthLog) -> Result<Self, Self::Error> {
        let first_topic = l.topics.get(0).cloned();
        let expected_topic = Some(*CANCELLED_USER_OP_TOPIC);

        if first_topic != expected_topic {
            return Err(Self::Error::WrongTopic {
                topic: first_topic.unwrap_or_default(),
            });
        };

        let zero_hash = EthHash::zero();
        let actor_address = EthAddress::from_slice(&l.topics.get(1).unwrap_or(&zero_hash)[12..]);
        let actor_type = ActorType::try_from(&U256::from_big_endian(l.topics.get(2).unwrap_or(&zero_hash).as_bytes()))?;
        let actor = Actor::new(actor_type, actor_address);
        debug!("actor parsed from cancellation log: {actor}");
        Ok(actor)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::user_ops::test_utils::get_log_with_protocol_cancellation_log;

    #[test]
    fn should_get_actor_from_cancellation_log_correctly() {
        let l = get_log_with_protocol_cancellation_log();
        let r = Actor::try_from(l).unwrap();
        let er = Actor::new(
            ActorType::Guardian,
            EthAddress::from_str("0xdb30d31ce9a22f36a44993b1079ad2d201e11788").unwrap(),
        );
        assert_eq!(r, er);
    }
}
