use std::fmt;

use common::BridgeSide;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
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
}
