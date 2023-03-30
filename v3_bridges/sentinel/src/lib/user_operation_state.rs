use std::fmt;

use common::BridgeSide;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum UserOpState {
    Witnessed(BridgeSide) = 1,
    Enqueued(BridgeSide) = 2,
    Executed(BridgeSide) = 3,
    Cancelled(BridgeSide) = 4,
}

impl fmt::Display for UserOpState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Enqueued(ref side) => write!(f, "enqueued on {side}"),
            Self::Executed(ref side) => write!(f, "executed on {side}"),
            Self::Witnessed(ref side) => write!(f, "witnessed on {side}"),
            Self::Cancelled(ref side) => write!(f, "cancelled on {side}"),
        }
    }
}

impl Default for UserOpState {
    fn default() -> Self {
        Self::Witnessed(BridgeSide::Native)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_op_state_should_be_ordered() {
        assert!(UserOpState::Witnessed(BridgeSide::Native) < UserOpState::Witnessed(BridgeSide::Host));
        let s = BridgeSide::Native;
        assert!(UserOpState::Witnessed(s) < UserOpState::Enqueued(s));
        assert!(UserOpState::Enqueued(s) < UserOpState::Executed(s));
        assert!(UserOpState::Executed(s) < UserOpState::Cancelled(s));
    }
}
