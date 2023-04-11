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

// TODO Make a struct for the args for each enum field?

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Serialize, Deserialize, EnumIter)]
pub enum UserOpState {
    Witnessed(BridgeSide, EthHash) = 1,
    Enqueued(BridgeSide, EthHash) = 2,
    Executed(BridgeSide, EthHash) = 3,
    Cancelled(BridgeSide, EthHash) = 4,
}

impl fmt::Display for UserOpState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Enqueued(ref side, ref hash) => write!(f, "enqueued on {side} @ tx 0x{hash:x}"),
            Self::Executed(ref side, ref hash) => write!(f, "executed on {side} @ tx 0x{hash:x}"),
            Self::Witnessed(ref side, ref hash) => write!(f, "witnessed on {side} @ tx 0x{hash:x}"),
            Self::Cancelled(ref side, ref hash) => write!(f, "cancelled on {side} @ tx 0x{hash:x}"),
        }
    }
}

impl Default for UserOpState {
    fn default() -> Self {
        Self::Witnessed(BridgeSide::Native, EthHash::default())
    }
}

impl UserOpState {
    pub fn try_from_log(side: BridgeSide, tx_hash: EthHash, log: &EthLog) -> Result<Self, UserOpError> {
        if log.topics.is_empty() {
            return Err(UserOpError::NoTopics);
        };

        if log.topics[0] == *WITNESSED_USER_OP_TOPIC {
            Ok(Self::Witnessed(side, tx_hash))
        } else if log.topics[0] == *ENQUEUED_USER_OP_TOPIC {
            Ok(Self::Enqueued(side, tx_hash))
        } else if log.topics[0] == *EXECUTED_USER_OP_TOPIC {
            Ok(Self::Executed(side, tx_hash))
        } else if log.topics[0] == *CANCELLED_USER_OP_TOPIC {
            Ok(Self::Cancelled(side, tx_hash))
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

    pub fn update(self, tx_hash: EthHash) -> Result<(Self, Self), UserOpError> {
        match self {
            Self::Witnessed(side, _) => Ok((self, Self::Enqueued(side, tx_hash))),
            Self::Enqueued(side, _) => Ok((self, Self::Executed(side, tx_hash))),
            op_state => Err(UserOpError::CannotUpdate(op_state)),
        }
    }

    pub fn cancel(self, tx_hash: EthHash) -> Result<(Self, Self), UserOpError> {
        match self {
            Self::Witnessed(side, _) => Ok((self, Self::Cancelled(side, tx_hash))),
            Self::Enqueued(side, _) => Ok((self, Self::Cancelled(side, tx_hash))),
            op_state => Err(UserOpError::CannotCancel(op_state)),
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
        match self {
            Self::Cancelled(..) => true,
            _ => false,
        }
    }

    pub fn is_executed(&self) -> bool {
        match self {
            Self::Executed(..) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_op_state_should_be_ordered() {
        let h = EthHash::default();
        assert!(UserOpState::Witnessed(BridgeSide::Native, h) < UserOpState::Witnessed(BridgeSide::Host, h));
        let s = BridgeSide::Native;
        assert!(UserOpState::Witnessed(s, h) < UserOpState::Enqueued(s, h));
        assert!(UserOpState::Enqueued(s, h) < UserOpState::Executed(s, h));
        assert!(UserOpState::Executed(s, h) < UserOpState::Cancelled(s, h));
    }

    #[test]
    fn should_update_user_op_state() {
        let side = BridgeSide::Native;
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::Witnessed(side, hash_1);
        let hash_2 = EthHash::random();
        let (prev, result) = user_op_state.update(hash_2).unwrap();
        assert_eq!(prev, user_op_state);
        let expected_result = UserOpState::Enqueued(side, hash_2);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_update_user_op_state() {
        let side = BridgeSide::Native;
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::Executed(side, hash_1);
        let hash_2 = EthHash::random();
        match user_op_state.update(hash_2) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotUpdate(e)) => assert_eq!(e, user_op_state),
            Err(e) => panic!("wrong error received: {e}"),
        }
    }

    #[test]
    fn should_cancel_user_op_state() {
        let side = BridgeSide::Native;
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::Witnessed(side, hash_1);
        let hash_2 = EthHash::random();
        let (prev, result) = user_op_state.cancel(hash_2).unwrap();
        assert_eq!(prev, user_op_state);
        let expected_result = UserOpState::Cancelled(side, hash_2);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_cancel_user_op_state() {
        let side = BridgeSide::Native;
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::Executed(side, hash_1);
        let hash_2 = EthHash::random();
        match user_op_state.cancel(hash_2) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotCancel(e)) => assert_eq!(e, user_op_state),
            Err(e) => panic!("wrong error received: {e}"),
        };
    }

    #[test]
    fn should_have_stateful_equality() {
        let h_1 = EthHash::random();
        let h_2 = EthHash::random();
        let b_1 = BridgeSide::Native;
        let b_2 = BridgeSide::Host;
        let a = UserOpState::Witnessed(b_1, h_1);
        let b = UserOpState::Witnessed(b_2, h_2);
        assert_ne!(a, b);
        assert!(a.is_same_state_as(b));
        assert!(a <= b);
        assert!(!(a > b));
    }
}
