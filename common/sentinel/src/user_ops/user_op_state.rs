use std::{cmp, fmt};

use common_eth::EthLog;
use derive_getters::Getters;
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, H256 as EthHash};
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
use crate::{get_utc_timestamp, Actor, ActorType, NetworkId};

lazy_static! {
    // NOTE: The zero address is a standin for _this_ sentinel, since we don't easily
    // have access to that key at runtime. Later the sentinel will witness the mining
    // of it's own cancellation tx which will contain the correct address.
    static ref SENTINEL_ACTOR: Actor = Actor::new(ActorType::Sentinel, EthAddress::zero());
}

#[serde_as]
#[derive(Debug, Default, Clone, Eq, PartialEq, Constructor, Deref, Serialize, Deserialize)]
pub struct UserOpStateInfos(Vec<UserOpStateInfo>);

#[serde_as]
#[derive(Debug, Default, Copy, Clone, Eq, Getters, Serialize, Deserialize)]
pub struct UserOpStateInfo {
    tx_hash: EthHash,
    #[serde_as(as = "DisplayFromStr")]
    network_id: NetworkId,
    sentinel_timestamp: u64,
    #[getter(skip)]
    block_timestamp: Option<u64>,
}

impl UserOpStateInfo {
    pub fn new(tx_hash: EthHash, network_id: NetworkId, block_timestamp: u64) -> Self {
        Self {
            tx_hash,
            network_id,
            block_timestamp: Some(block_timestamp),
            sentinel_timestamp: get_utc_timestamp().unwrap_or_default(),
        }
    }

    pub fn block_timestamp(&self) -> Result<u64, UserOpError> {
        self.block_timestamp.ok_or(UserOpError::NoBlockTimestampInUserOpState)
    }
}

impl PartialEq for UserOpStateInfo {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We don't care about the timestamps when comparing these...
        self.tx_hash == other.tx_hash && self.network_id == other.network_id
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Constructor, Deref, DerefMut)]
pub struct UserOpStates(Vec<UserOpState>);

impl UserOpStates {
    pub fn empty() -> Self {
        Self::default()
    }
}

impl Default for UserOpStates {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, EnumIter)]
pub enum UserOpState {
    Witnessed(UserOpStateInfo) = 1,
    Enqueued(UserOpStateInfo) = 2,
    Executed(UserOpStateInfo) = 3,
    Cancelled(UserOpStateInfo, Actor) = 4,
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
    pub fn witnessed(nid: NetworkId, h: EthHash, block_timestamp: u64) -> Self {
        Self::Witnessed(UserOpStateInfo::new(h, nid, block_timestamp))
    }

    pub fn enqueued(nid: NetworkId, h: EthHash, block_timestamp: u64) -> Self {
        Self::Enqueued(UserOpStateInfo::new(h, nid, block_timestamp))
    }

    pub fn executed(nid: NetworkId, h: EthHash, block_timestamp: u64) -> Self {
        Self::Executed(UserOpStateInfo::new(h, nid, block_timestamp))
    }

    pub fn cancelled(nid: NetworkId, h: EthHash, block_timestamp: u64) -> Self {
        let actor = Actor::new(ActorType::Sentinel, EthAddress::zero());
        Self::Cancelled(UserOpStateInfo::new(h, nid, block_timestamp), actor)
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
            block_timestamp,
        } = self.state();
        write!(
            f,
            "{x} @ tx 0x{:x} @ sentinel time {}, w/ block timestamp {}, nid: {}",
            tx_hash,
            sentinel_timestamp,
            block_timestamp.unwrap_or_default(),
            network_id,
        )
    }
}

impl Default for UserOpState {
    fn default() -> Self {
        Self::Witnessed(UserOpStateInfo::default())
    }
}

impl From<Vec<UserOpState>> for UserOpStateInfos {
    fn from(ss: Vec<UserOpState>) -> Self {
        UserOpStateInfos::new(ss.iter().map(|s| s.user_op_state_info()).collect::<Vec<_>>())
    }
}

impl UserOpState {
    pub fn block_timestamp(&self) -> Result<u64, UserOpError> {
        match self {
            Self::Enqueued(state, ..) => state.block_timestamp(),
            Self::Executed(state, ..) => state.block_timestamp(),
            Self::Witnessed(state, ..) => state.block_timestamp(),
            Self::Cancelled(state, ..) => state.block_timestamp(),
        }
    }

    pub fn network_id(&self) -> NetworkId {
        match self {
            Self::Enqueued(state, ..) => *state.network_id(),
            Self::Executed(state, ..) => *state.network_id(),
            Self::Witnessed(state, ..) => *state.network_id(),
            Self::Cancelled(state, ..) => *state.network_id(),
        }
    }

    fn state(&self) -> UserOpStateInfo {
        match self {
            Self::Witnessed(ref state) => *state,
            Self::Enqueued(ref state) => *state,
            Self::Executed(ref state) => *state,
            Self::Cancelled(ref state, _) => *state,
        }
    }

    fn user_op_state_info(&self) -> UserOpStateInfo {
        self.state()
    }

    pub fn try_from_log(
        nid: NetworkId,
        tx_hash: EthHash,
        block_timestamp: u64,
        log: &EthLog,
    ) -> Result<Self, UserOpError> {
        if log.topics.is_empty() {
            return Err(UserOpError::NoTopics);
        };

        let state = UserOpStateInfo::new(tx_hash, nid, block_timestamp);

        if log.topics[0] == *WITNESSED_USER_OP_TOPIC {
            Ok(Self::Witnessed(state))
        } else if log.topics[0] == *ENQUEUED_USER_OP_TOPIC {
            Ok(Self::Enqueued(state))
        } else if log.topics[0] == *EXECUTED_USER_OP_TOPIC {
            Ok(Self::Executed(state))
        } else if log.topics[0] == *CANCELLED_USER_OP_TOPIC {
            let actor = Actor::try_from(log)?;
            Ok(Self::Cancelled(state, actor))
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

    pub fn update(self, tx_hash: EthHash, block_timestamp: u64) -> Result<(Self, Self), UserOpError> {
        match self {
            Self::Witnessed(UserOpStateInfo { network_id, .. }) => Ok((
                self,
                Self::Enqueued(UserOpStateInfo::new(tx_hash, network_id, block_timestamp)),
            )),
            Self::Enqueued(UserOpStateInfo { network_id, .. }) => Ok((
                self,
                Self::Executed(UserOpStateInfo::new(tx_hash, network_id, block_timestamp)),
            )),
            op_state => Err(UserOpError::CannotUpdate {
                from: Box::new(op_state),
                to: Box::new(UserOpState::Cancelled(
                    UserOpStateInfo::new(tx_hash, op_state.nid(), block_timestamp),
                    *SENTINEL_ACTOR,
                )),
            }),
        }
    }

    pub fn cancel(self, tx_hash: EthHash, block_timestamp: u64) -> Result<(Self, Self), UserOpError> {
        match self {
            Self::Witnessed(UserOpStateInfo { network_id, .. })
            | Self::Enqueued(UserOpStateInfo { network_id, .. }) => Ok((
                self,
                Self::Cancelled(
                    UserOpStateInfo::new(tx_hash, network_id, block_timestamp),
                    *SENTINEL_ACTOR,
                ),
            )),
            op_state => Err(UserOpError::CannotCancelOpInState(op_state)),
        }
    }

    pub fn nid(&self) -> NetworkId {
        match self {
            Self::Witnessed(UserOpStateInfo { network_id, .. }) => *network_id,
            Self::Enqueued(UserOpStateInfo { network_id, .. }) => *network_id,
            Self::Executed(UserOpStateInfo { network_id, .. }) => *network_id,
            Self::Cancelled(UserOpStateInfo { network_id, .. }, _) => *network_id,
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
        let block_timestamp = 0;
        let h = EthHash::default();
        let nid = NetworkId::default();
        assert!(UserOpState::witnessed(nid, h, block_timestamp) == UserOpState::witnessed(nid, h, block_timestamp));
        assert!(UserOpState::witnessed(nid, h, block_timestamp) < UserOpState::enqueued(nid, h, block_timestamp));
        assert!(UserOpState::enqueued(nid, h, block_timestamp) < UserOpState::executed(nid, h, block_timestamp));
        assert!(UserOpState::executed(nid, h, block_timestamp) < UserOpState::cancelled(nid, h, block_timestamp));
    }

    #[test]
    fn should_update_user_op_state() {
        let block_timestamp = 0;
        let nid = NetworkId::default();
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::witnessed(nid, hash_1, block_timestamp);
        let hash_2 = EthHash::random();
        let (prev, result) = user_op_state.update(hash_2, block_timestamp).unwrap();
        assert_eq!(prev, user_op_state);
        let expected_result = UserOpState::enqueued(nid, hash_2, block_timestamp);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_update_user_op_state() {
        let block_timestamp = 0;
        let nid = NetworkId::default();
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::executed(nid, hash_1, block_timestamp);
        let hash_2 = EthHash::random();
        let actor = Actor::new(ActorType::Sentinel, EthAddress::zero());
        match user_op_state.update(hash_2, block_timestamp) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotUpdate { from, to }) => {
                assert_eq!(from, Box::new(user_op_state));
                assert_eq!(
                    to,
                    Box::new(UserOpState::Cancelled(
                        UserOpStateInfo::new(hash_2, nid, block_timestamp),
                        actor
                    ))
                );
            },
            Err(e) => panic!("wrong error received: {e}"),
        }
    }

    #[test]
    fn should_cancel_user_op_state() {
        let block_timestamp = 0;
        let nid = NetworkId::default();
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::witnessed(nid, hash_1, block_timestamp);
        let hash_2 = EthHash::random();
        let (prev, result) = user_op_state.cancel(hash_2, block_timestamp).unwrap();
        assert_eq!(prev, user_op_state);
        let expected_result = UserOpState::cancelled(nid, hash_2, block_timestamp);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_cancel_user_op_state() {
        let block_timestamp = 0;
        let nid = NetworkId::default();
        let hash_1 = EthHash::random();
        let user_op_state = UserOpState::executed(nid, hash_1, block_timestamp);
        let hash_2 = EthHash::random();
        match user_op_state.cancel(hash_2, block_timestamp) {
            Ok(_) => panic!("should not have succeeded!"),
            Err(UserOpError::CannotCancelOpInState(e)) => assert_eq!(e, user_op_state),
            Err(e) => panic!("wrong error received: {e}"),
        };
    }

    #[test]
    fn should_have_stateful_equality() {
        let block_timestamp = 0;
        let h_1 = EthHash::random();
        let h_2 = EthHash::random();
        let b_1 = NetworkId::default();
        let b_2 = NetworkId::default();
        let a = UserOpState::witnessed(b_1, h_1, block_timestamp);
        let b = UserOpState::witnessed(b_2, h_2, block_timestamp);
        assert_ne!(a, b);
        assert!(a.is_same_state_as(b));
        assert!(a <= b);
        assert!(!(a > b));
    }
}
