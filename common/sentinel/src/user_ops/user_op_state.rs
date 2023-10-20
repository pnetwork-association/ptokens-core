use std::{cmp, fmt};

use common_eth::EthLog;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use super::{
    UserOpError,
    CANCELLED_USER_OP_TOPIC,
    ENQUEUED_USER_OP_TOPIC,
    EXECUTED_USER_OP_TOPIC,
    WITNESSED_USER_OP_TOPIC,
};
use crate::{get_utc_timestamp, NetworkId};

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, Serialize, Deserialize, EnumIter)]
pub enum UserOpState {
    Witnessed(NetworkId, EthHash, u64) = 1,
    Enqueued(NetworkId, EthHash, u64) = 2,
    Executed(NetworkId, EthHash, u64) = 3,
    Cancelled(NetworkId, EthHash, u64) = 4,
}

impl From<&UserOpState> for u8 {
    fn from(s: &UserOpState) -> u8 {
        match s {
            UserOpState::Witnessed(..) => 1,
            UserOpState::Enqueued(..) => 2,
            UserOpState::Executed(..) => 3,
            UserOpState::Cancelled(..) => 4,
        }
    }
}

impl PartialOrd for UserOpState {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserOpState {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let s: u8 = self.into();
        let o: u8 = other.into();
        s.cmp(&o)
    }
}

impl PartialEq for UserOpState {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We don't care about the timestamps when comparing these...
        match (self, other) {
            (Self::Enqueued(nid_a, h_a, _), Self::Enqueued(nid_b, h_b, _))
            | (Self::Executed(nid_a, h_a, _), Self::Executed(nid_b, h_b, _))
            | (Self::Witnessed(nid_a, h_a, _), Self::Witnessed(nid_b, h_b, _))
            | (Self::Cancelled(nid_a, h_a, _), Self::Cancelled(nid_b, h_b, _)) => nid_a == nid_b && h_a == h_b,
            _ => false,
        }
    }
}

#[cfg(test)]
impl UserOpState {
    pub fn witnessed(nid: NetworkId, h: EthHash) -> Self {
        Self::Witnessed(nid, h, get_utc_timestamp().unwrap_or_default())
    }

    pub fn enqueued(nid: NetworkId, h: EthHash) -> Self {
        Self::Enqueued(nid, h, get_utc_timestamp().unwrap_or_default())
    }

    pub fn executed(nid: NetworkId, h: EthHash) -> Self {
        Self::Executed(nid, h, get_utc_timestamp().unwrap_or_default())
    }

    pub fn cancelled(nid: NetworkId, h: EthHash) -> Self {
        Self::Cancelled(nid, h, get_utc_timestamp().unwrap_or_default())
    }
}

impl fmt::Display for UserOpState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Enqueued(ref nid, ref hash, ref timestamp) => {
                write!(f, "enqueued @ tx 0x{hash:x} @ time {timestamp}, nid: {nid}")
            },
            Self::Executed(ref nid, ref hash, ref timestamp) => {
                write!(f, "executed @ tx 0x{hash:x} @ time {timestamp}, nid: {nid}")
            },
            Self::Witnessed(ref nid, ref hash, ref timestamp) => {
                write!(f, "witnessed @ tx 0x{hash:x} @ time {timestamp}, nid: {nid}")
            },
            Self::Cancelled(ref nid, ref hash, ref timestamp) => {
                write!(f, "cancelled @ tx 0x{hash:x} @ time {timestamp}, nid: {nid}")
            },
        }
    }
}

impl Default for UserOpState {
    fn default() -> Self {
        Self::Witnessed(NetworkId::default(), EthHash::default(), <u64>::default())
    }
}

impl UserOpState {
    pub(super) fn timestamp(&self) -> u64 {
        match self {
            Self::Witnessed(_, _, timestamp) => *timestamp,
            Self::Enqueued(_, _, timestamp) => *timestamp,
            Self::Executed(_, _, timestamp) => *timestamp,
            Self::Cancelled(_, _, timestamp) => *timestamp,
        }
    }

    pub fn try_from_log(nid: NetworkId, tx_hash: EthHash, log: &EthLog, timestamp: u64) -> Result<Self, UserOpError> {
        if log.topics.is_empty() {
            return Err(UserOpError::NoTopics);
        };

        if log.topics[0] == *WITNESSED_USER_OP_TOPIC {
            Ok(Self::Witnessed(nid, tx_hash, timestamp))
        } else if log.topics[0] == *ENQUEUED_USER_OP_TOPIC {
            Ok(Self::Enqueued(nid, tx_hash, timestamp))
        } else if log.topics[0] == *EXECUTED_USER_OP_TOPIC {
            Ok(Self::Executed(nid, tx_hash, timestamp))
        } else if log.topics[0] == *CANCELLED_USER_OP_TOPIC {
            Ok(Self::Cancelled(nid, tx_hash, timestamp))
        } else {
            Err(UserOpError::UnrecognizedTopic(log.topics[0]))
        }
    }

    #[rustfmt::skip]
    pub fn is_same_state_as(&self, other: Self) -> bool {
        // NOTE: The derived == allows for a strict equality, whereas this method allows us to
        // check equality of the state and nothing else.
        matches!(
            (self, other),
            (Self::Witnessed(..), Self::Witnessed(..)) |
            (Self::Enqueued(..), Self::Enqueued(..)) |
            (Self::Executed(..), Self::Executed(..)) |
            (Self::Cancelled(..), Self::Cancelled(..))
        )
    }

    pub fn update(self, tx_hash: EthHash, timestamp: u64) -> Result<(Self, Self), UserOpError> {
        match self {
            Self::Witnessed(nid, ..) => Ok((self, Self::Enqueued(nid, tx_hash, timestamp))),
            Self::Enqueued(nid, ..) => Ok((self, Self::Executed(nid, tx_hash, timestamp))),
            op_state => Err(UserOpError::CannotUpdate {
                from: Box::new(op_state),
                to: Box::new(UserOpState::Cancelled(op_state.nid(), tx_hash, timestamp)),
            }),
        }
    }

    pub fn cancel(self, tx_hash: EthHash) -> Result<(Self, Self), UserOpError> {
        match self {
            Self::Witnessed(nid, ..) => Ok((self, Self::Cancelled(nid, tx_hash, get_utc_timestamp()?))),
            Self::Enqueued(nid, ..) => Ok((self, Self::Cancelled(nid, tx_hash, get_utc_timestamp()?))),
            op_state => Err(UserOpError::CannotCancelOpInState(op_state)),
        }
    }

    pub fn nid(&self) -> NetworkId {
        match self {
            Self::Witnessed(nid, ..) => *nid,
            Self::Enqueued(nid, ..) => *nid,
            Self::Executed(nid, ..) => *nid,
            Self::Cancelled(nid, ..) => *nid,
        }
    }

    pub fn get_bit_flag_idx(&self) -> u8 {
        match self {
            Self::Witnessed(..) => 0,
            Self::Enqueued(..) => 1,
            Self::Executed(..) => 2,
            Self::Cancelled(..) => 3,
        }
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled(..))
    }

    pub fn is_executed(&self) -> bool {
        matches!(self, Self::Executed(..))
    }

    pub fn is_witnessed(&self) -> bool {
        matches!(self, Self::Witnessed(..))
    }

    pub fn is_enqueued(&self) -> bool {
        matches!(self, Self::Enqueued(..))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_op_state_should_be_ordered() {
        let h = EthHash::default();
        let nid = NetworkId::default();
        assert!(UserOpState::witnessed(nid, h) == UserOpState::witnessed(nid, h));
        assert!(UserOpState::witnessed(nid, h) < UserOpState::enqueued(nid, h));
        assert!(UserOpState::enqueued(nid, h) < UserOpState::executed(nid, h));
        assert!(UserOpState::executed(nid, h) < UserOpState::cancelled(nid, h));
    }

    #[test]
    fn should_update_user_op_state() {
        let nid = NetworkId::default();
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::witnessed(nid, hash_1);
        let hash_2 = EthHash::random();
        let (prev, result) = user_op_state.update(hash_2, 1).unwrap();
        assert_eq!(prev, user_op_state);
        let expected_result = UserOpState::enqueued(nid, hash_2);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_update_user_op_state() {
        let nid = NetworkId::default();
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::executed(nid, hash_1);
        let hash_2 = EthHash::random();
        match user_op_state.update(hash_2, 1) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotUpdate { from, to }) => {
                assert_eq!(from, Box::new(user_op_state));
                assert_eq!(to, Box::new(UserOpState::Cancelled(nid, hash_2, 1)));
            },
            Err(e) => panic!("wrong error received: {e}"),
        }
    }

    #[test]
    fn should_cancel_user_op_state() {
        let nid = NetworkId::default();
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::witnessed(nid, hash_1);
        let hash_2 = EthHash::random();
        let (prev, result) = user_op_state.cancel(hash_2).unwrap();
        assert_eq!(prev, user_op_state);
        let expected_result = UserOpState::cancelled(nid, hash_2);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_cancel_user_op_state() {
        let nid = NetworkId::default();
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::executed(nid, hash_1);
        let hash_2 = EthHash::random();
        match user_op_state.cancel(hash_2) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotCancelOpInState(e)) => assert_eq!(e, user_op_state),
            Err(e) => panic!("wrong error received: {e}"),
        };
    }

    #[test]
    fn should_have_stateful_equality() {
        let h_1 = EthHash::random();
        let h_2 = EthHash::random();
        let b_1 = NetworkId::default();
        let b_2 = NetworkId::default();
        let a = UserOpState::witnessed(b_1, h_1);
        let b = UserOpState::witnessed(b_2, h_2);
        assert_ne!(a, b);
        assert!(a.is_same_state_as(b));
        assert!(a <= b);
        assert!(!(a > b));
    }
}
