use std::{cmp, fmt};

use common_eth::EthLog;
use derive_getters::Getters;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use strum_macros::EnumIter;

use super::{
    UserOpError,
    CANCELLED_USER_OP_TOPIC,
    ENQUEUED_USER_OP_TOPIC,
    EXECUTED_USER_OP_TOPIC,
    WITNESSED_USER_OP_TOPIC,
};
use crate::{get_utc_timestamp, NetworkId};

#[serde_as]
#[derive(Debug, Default, Copy, Clone, Eq, Getters, Serialize, Deserialize)]
pub struct UserOpStateInfo {
    tx_hash: EthHash,
    #[serde_as(as = "DisplayFromStr")]
    network_id: NetworkId,
    sentinel_timestamp: u64,
}

impl UserOpStateInfo {
    pub fn new(tx_hash: EthHash, network_id: NetworkId) -> Self {
        Self {
            tx_hash,
            network_id,
            sentinel_timestamp: get_utc_timestamp().unwrap_or_default(),
        }
    }
}

impl PartialEq for UserOpStateInfo {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We don't care about the timestamps when comparing these...
        self.tx_hash == other.tx_hash && self.network_id == other.network_id
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, EnumIter)]
pub enum UserOpState {
    Witnessed(UserOpStateInfo) = 1,
    Enqueued(UserOpStateInfo) = 2,
    Executed(UserOpStateInfo) = 3,
    Cancelled(UserOpStateInfo) = 4,
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

#[cfg(test)]
impl UserOpState {
    pub fn witnessed(nid: NetworkId, h: EthHash) -> Self {
        Self::Witnessed(UserOpStateInfo::new(h, nid))
    }

    pub fn enqueued(nid: NetworkId, h: EthHash) -> Self {
        Self::Enqueued(UserOpStateInfo::new(h, nid))
    }

    pub fn executed(nid: NetworkId, h: EthHash) -> Self {
        Self::Executed(UserOpStateInfo::new(h, nid))
    }

    pub fn cancelled(nid: NetworkId, h: EthHash) -> Self {
        Self::Cancelled(UserOpStateInfo::new(h, nid))
    }
}

impl fmt::Display for UserOpState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = match self {
            Self::Enqueued(..) => "enqueued",
            Self::Executed(..) => "executed",
            Self::Witnessed(..) => "witnessed",
            Self::Cancelled(..) => "cancelled",
        };
        let UserOpStateInfo {
            network_id,
            tx_hash,
            sentinel_timestamp,
        } = self.state();
        write!(
            f,
            "{x} @ tx 0x{tx_hash:x} @ time {sentinel_timestamp}, nid: {network_id}"
        )
    }
}

impl Default for UserOpState {
    fn default() -> Self {
        Self::Witnessed(UserOpStateInfo::default())
    }
}

impl UserOpState {
    fn state(&self) -> UserOpStateInfo {
        match self {
            Self::Witnessed(ref state) => *state,
            Self::Enqueued(ref state) => *state,
            Self::Executed(ref state) => *state,
            Self::Cancelled(ref state) => *state,
        }
    }

    pub(super) fn timestamp(&self) -> u64 {
        match self {
            Self::Witnessed(UserOpStateInfo { sentinel_timestamp, .. }) => *sentinel_timestamp,
            Self::Enqueued(UserOpStateInfo { sentinel_timestamp, .. }) => *sentinel_timestamp,
            Self::Executed(UserOpStateInfo { sentinel_timestamp, .. }) => *sentinel_timestamp,
            Self::Cancelled(UserOpStateInfo { sentinel_timestamp, .. }) => *sentinel_timestamp,
        }
    }

    pub fn try_from_log(nid: NetworkId, tx_hash: EthHash, log: &EthLog) -> Result<Self, UserOpError> {
        if log.topics.is_empty() {
            return Err(UserOpError::NoTopics);
        };

        let state = UserOpStateInfo::new(tx_hash, nid);

        if log.topics[0] == *WITNESSED_USER_OP_TOPIC {
            Ok(Self::Witnessed(state))
        } else if log.topics[0] == *ENQUEUED_USER_OP_TOPIC {
            Ok(Self::Enqueued(state))
        } else if log.topics[0] == *EXECUTED_USER_OP_TOPIC {
            Ok(Self::Executed(state))
        } else if log.topics[0] == *CANCELLED_USER_OP_TOPIC {
            Ok(Self::Cancelled(state))
        } else {
            Err(UserOpError::UnrecognizedTopic(log.topics[0]))
        }
    }

    #[rustfmt::skip]
    pub fn is_same_state_as(&self, other: Self) -> bool {
        // NOTE: The derived == allows for a strict equality, whereas this method allows us to
        // check equality of the enum state and nothing else.
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
            Self::Witnessed(UserOpStateInfo { network_id, .. }) => {
                Ok((self, Self::Enqueued(UserOpStateInfo::new(tx_hash, network_id))))
            },
            Self::Enqueued(UserOpStateInfo { network_id, .. }) => {
                Ok((self, Self::Executed(UserOpStateInfo::new(tx_hash, network_id))))
            },
            op_state => Err(UserOpError::CannotUpdate {
                from: Box::new(op_state),
                to: Box::new(UserOpState::Cancelled(UserOpStateInfo::new(tx_hash, op_state.nid()))),
            }),
        }
    }

    pub fn cancel(self, tx_hash: EthHash) -> Result<(Self, Self), UserOpError> {
        match self {
            Self::Witnessed(UserOpStateInfo { network_id, .. }) => {
                Ok((self, Self::Cancelled(UserOpStateInfo::new(tx_hash, network_id))))
            },
            Self::Enqueued(UserOpStateInfo { network_id, .. }) => {
                Ok((self, Self::Cancelled(UserOpStateInfo::new(tx_hash, network_id))))
            },
            op_state => Err(UserOpError::CannotCancelOpInState(op_state)),
        }
    }

    pub fn nid(&self) -> NetworkId {
        match self {
            Self::Witnessed(UserOpStateInfo { network_id, .. }) => *network_id,
            Self::Enqueued(UserOpStateInfo { network_id, .. }) => *network_id,
            Self::Executed(UserOpStateInfo { network_id, .. }) => *network_id,
            Self::Cancelled(UserOpStateInfo { network_id, .. }) => *network_id,
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
        let (prev, result) = user_op_state.update(hash_2).unwrap();
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
        match user_op_state.update(hash_2) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotUpdate { from, to }) => {
                assert_eq!(from, Box::new(user_op_state));
                assert_eq!(to, Box::new(UserOpState::Cancelled(UserOpStateInfo::new(hash_2, nid))));
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
