use std::fmt;

use common::BridgeSide;
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
use crate::get_utc_timestamp;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialOrd, Serialize, Deserialize, EnumIter)]
pub enum UserOpState {
    Witnessed(BridgeSide, EthHash, u64) = 1,
    Enqueued(BridgeSide, EthHash, u64) = 2,
    Executed(BridgeSide, EthHash, u64) = 3,
    Cancelled(BridgeSide, EthHash, u64) = 4,
}

impl PartialEq for UserOpState {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We don't care about the timestamps when comparing these...
        match (self, other) {
            (Self::Enqueued(s_a, h_a, _), Self::Enqueued(s_b, h_b, _))
            | (Self::Executed(s_a, h_a, _), Self::Executed(s_b, h_b, _))
            | (Self::Witnessed(s_a, h_a, _), Self::Witnessed(s_b, h_b, _))
            | (Self::Cancelled(s_a, h_a, _), Self::Cancelled(s_b, h_b, _)) => s_a == s_b && h_a == h_b,
            _ => false,
        }
    }
}

#[cfg(test)]
impl UserOpState {
    pub fn witnessed(s: BridgeSide, h: EthHash) -> Self {
        Self::Witnessed(s, h, get_utc_timestamp().unwrap_or_default())
    }

    pub fn enqueued(s: BridgeSide, h: EthHash) -> Self {
        Self::Enqueued(s, h, get_utc_timestamp().unwrap_or_default())
    }

    pub fn executed(s: BridgeSide, h: EthHash) -> Self {
        Self::Executed(s, h, get_utc_timestamp().unwrap_or_default())
    }

    pub fn cancelled(s: BridgeSide, h: EthHash) -> Self {
        Self::Cancelled(s, h, get_utc_timestamp().unwrap_or_default())
    }
}

impl fmt::Display for UserOpState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Enqueued(ref side, ref hash, ref timestamp) => {
                write!(f, "enqueued on {side} @ tx 0x{hash:x} @ time {timestamp}")
            },
            Self::Executed(ref side, ref hash, ref timestamp) => {
                write!(f, "executed on {side} @ tx 0x{hash:x} @ time {timestamp}")
            },
            Self::Witnessed(ref side, ref hash, ref timestamp) => {
                write!(f, "witnessed on {side} @ tx 0x{hash:x} @ time {timestamp}")
            },
            Self::Cancelled(ref side, ref hash, ref timestamp) => {
                write!(f, "cancelled on {side} @ tx 0x{hash:x} @ time {timestamp}")
            },
        }
    }
}

impl Default for UserOpState {
    fn default() -> Self {
        Self::Witnessed(BridgeSide::Native, EthHash::default(), <u64>::default())
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

    pub fn try_from_log(side: BridgeSide, tx_hash: EthHash, log: &EthLog, timestamp: u64) -> Result<Self, UserOpError> {
        if log.topics.is_empty() {
            return Err(UserOpError::NoTopics);
        };

        if log.topics[0] == *WITNESSED_USER_OP_TOPIC {
            Ok(Self::Witnessed(side, tx_hash, timestamp))
        } else if log.topics[0] == *ENQUEUED_USER_OP_TOPIC {
            Ok(Self::Enqueued(side, tx_hash, timestamp))
        } else if log.topics[0] == *EXECUTED_USER_OP_TOPIC {
            Ok(Self::Executed(side, tx_hash, timestamp))
        } else if log.topics[0] == *CANCELLED_USER_OP_TOPIC {
            Ok(Self::Cancelled(side, tx_hash, timestamp))
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
            Self::Witnessed(side, ..) => Ok((self, Self::Enqueued(side, tx_hash, timestamp))),
            Self::Enqueued(side, ..) => Ok((self, Self::Executed(side, tx_hash, timestamp))),
            op_state => Err(UserOpError::CannotUpdate {
                from: op_state,
                to: UserOpState::Cancelled(op_state.side(), tx_hash, timestamp),
            }),
        }
    }

    pub fn cancel(self, tx_hash: EthHash) -> Result<(Self, Self), UserOpError> {
        match self {
            Self::Witnessed(side, ..) => Ok((self, Self::Cancelled(side, tx_hash, get_utc_timestamp()?))),
            Self::Enqueued(side, ..) => Ok((self, Self::Cancelled(side, tx_hash, get_utc_timestamp()?))),
            op_state => Err(UserOpError::CannotCancelOpInState(op_state)),
        }
    }

    pub fn side(&self) -> BridgeSide {
        match self {
            Self::Witnessed(side, ..) => *side,
            Self::Enqueued(side, ..) => *side,
            Self::Executed(side, ..) => *side,
            Self::Cancelled(side, ..) => *side,
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

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_op_state_should_be_ordered() {
        let h = EthHash::default();
        assert!(UserOpState::witnessed(BridgeSide::Native, h) < UserOpState::witnessed(BridgeSide::Host, h));
        let s = BridgeSide::Native;
        assert!(UserOpState::witnessed(s, h) < UserOpState::enqueued(s, h));
        assert!(UserOpState::enqueued(s, h) < UserOpState::executed(s, h));
        assert!(UserOpState::executed(s, h) < UserOpState::cancelled(s, h));
    }

    #[test]
    fn should_update_user_op_state() {
        let side = BridgeSide::Native;
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::witnessed(side, hash_1);
        let hash_2 = EthHash::random();
        let (prev, result) = user_op_state.update(hash_2, 1).unwrap();
        assert_eq!(prev, user_op_state);
        let expected_result = UserOpState::enqueued(side, hash_2);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_update_user_op_state() {
        let side = BridgeSide::Native;
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::executed(side, hash_1);
        let hash_2 = EthHash::random();
        match user_op_state.update(hash_2, 1) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotUpdate { from, to }) => {
                assert_eq!(from, user_op_state);
                assert_eq!(to, UserOpState::Cancelled(side, hash_2, 1));
            },
            Err(e) => panic!("wrong error received: {e}"),
        }
    }

    #[test]
    fn should_cancel_user_op_state() {
        let side = BridgeSide::Native;
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::witnessed(side, hash_1);
        let hash_2 = EthHash::random();
        let (prev, result) = user_op_state.cancel(hash_2).unwrap();
        assert_eq!(prev, user_op_state);
        let expected_result = UserOpState::cancelled(side, hash_2);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_cancel_user_op_state() {
        let side = BridgeSide::Native;
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::executed(side, hash_1);
        let hash_2 = EthHash::random();
        match user_op_state.cancel(hash_2) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotCancel(e)) => assert_eq!(e, Box::new(user_op_state)),
            Err(e) => panic!("wrong error received: {e}"),
        };
    }

    #[test]
    fn should_have_stateful_equality() {
        let h_1 = EthHash::random();
        let h_2 = EthHash::random();
        let b_1 = BridgeSide::Native;
        let b_2 = BridgeSide::Host;
        let a = UserOpState::witnessed(b_1, h_1);
        let b = UserOpState::witnessed(b_2, h_2);
        assert_ne!(a, b);
        assert!(a.is_same_state_as(b));
        assert!(a <= b);
        assert!(!(a > b));
    }
}
*/
