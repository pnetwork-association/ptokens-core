use std::fmt;

use common::{BridgeSide, Byte};
use derive_more::{Constructor, Deref};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{UserOp, UserOpState};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Deref, Constructor, Serialize, Deserialize)]
pub struct UserOpFlag(Byte);

impl fmt::Display for UserOpFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = if self.is_witnessed() {
            "witnessed"
        } else if self.is_enqueued() {
            "enqueued"
        } else if self.is_executed() {
            "executed"
        } else if self.is_cancelled() {
            "cancelled"
        } else {
            "not set"
        };
        write!(f, "{s}")
    }
}

impl From<&UserOp> for UserOpFlag {
    fn from(op: &UserOp) -> Self {
        let mut s = Self::default();
        s.set_flag(&op.state());
        s
    }
}

impl From<&UserOpState> for UserOpFlag {
    fn from(state: &UserOpState) -> Self {
        let mut s = Self::default();
        s.set_flag(state);
        s
    }
}

impl From<&UserOpFlag> for UserOpState {
    fn from(flag: &UserOpFlag) -> Self {
        let n: u8 = **flag;
        let side = BridgeSide::default();
        let zero_hash = EthHash::default();

        let max_witnessed: u8 = 0b0000_0001;
        let max_enqueued: u8 = 0b0000_0011;
        let max_executed: u8 = 0b0000_0111;

        match n {
            _x if n <= max_witnessed => Self::Witnessed(side, zero_hash, <u64>::default()),
            _x if n <= max_enqueued => Self::Enqueued(side, zero_hash, <u64>::default()),
            _x if n <= max_executed => Self::Executed(side, zero_hash, <u64>::default()),
            _ => Self::Cancelled(side, zero_hash, <u64>::default()),
        }
    }
}

impl From<UserOpFlag> for UserOpState {
    fn from(flag: UserOpFlag) -> Self {
        Self::from(&flag)
    }
}

impl UserOpFlag {
    fn set_flag(&mut self, state: &UserOpState) {
        match state {
            UserOpState::Witnessed(..) => self.0 |= 0b0000_0001,
            UserOpState::Enqueued(..) => self.0 |= 0b0000_0010,
            UserOpState::Executed(..) => self.0 |= 0b0000_0100,
            UserOpState::Cancelled(..) => self.0 |= 0b0000_1000,
        }
    }

    #[allow(unused)]
    fn is_set(&self, state: &UserOpState) -> bool {
        self.bit_is_set(state.get_bit_flag_idx())
    }

    #[allow(unused)]
    fn clear_flag(mut self, state: &UserOpState) -> Self {
        match state {
            UserOpState::Witnessed(..) => self.0 &= 0b1111_1110,
            UserOpState::Enqueued(..) => self.0 &= 0b1111_1101,
            UserOpState::Executed(..) => self.0 &= 0b1111_1011,
            UserOpState::Cancelled(..) => self.0 &= 0b1111_0111,
        };
        self
    }

    fn bit_is_set(&self, n: u8) -> bool {
        if n > 7 {
            false
        } else {
            (self.0 & (1 << n)) != 0
        }
    }

    pub fn is_witnessed(&self) -> bool {
        self.bit_is_set(0)
    }

    pub fn is_enqueued(&self) -> bool {
        self.bit_is_set(1)
    }

    pub fn is_executed(&self) -> bool {
        self.bit_is_set(2)
    }

    pub fn is_cancelled(&self) -> bool {
        self.bit_is_set(3)
    }
}

#[cfg(test)]
mod tests {
    use common::BridgeSide;
    use ethereum_types::H256 as EthHash;
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn default_should_have_no_flag_set() {
        let user_op_flag = UserOpFlag::default();
        for n in 0..8 {
            assert!(!user_op_flag.bit_is_set(n))
        }
    }

    #[test]
    fn should_set_flag_for_each_state() {
        let states = UserOpState::iter().collect::<Vec<UserOpState>>();
        for state in states {
            let mut user_op_flag = UserOpFlag::default();
            assert!(!user_op_flag.is_set(&state));
            user_op_flag.set_flag(&state);
            assert!(user_op_flag.is_set(&state));
        }
    }

    #[test]
    fn should_not_overflow_when_shifting() {
        let user_op_flag = UserOpFlag::default();
        let n = 155;
        assert!(n > 8);
        let result = user_op_flag.bit_is_set(n);
        assert!(!result);
    }

    #[test]
    fn should_be_able_to_have_multiple_flag_set() {
        let mut user_op_flag = UserOpFlag::default();
        let side = BridgeSide::Native;
        let hash = EthHash::random();
        let s1 = UserOpState::Witnessed(side, hash);
        let s2 = UserOpState::Enqueued(side, hash);
        user_op_flag.set_flag(&s1);
        assert!(user_op_flag.is_set(&s1));
        user_op_flag.set_flag(&s2);
        assert!(user_op_flag.is_set(&s1));
        assert!(user_op_flag.is_set(&s2));
        assert!(user_op_flag.is_witnessed());
        assert!(user_op_flag.is_enqueued());
        assert!(!user_op_flag.is_executed());
        assert!(!user_op_flag.is_cancelled());
    }

    #[test]
    fn flags_should_be_orderable() {
        let states = UserOpState::iter().collect::<Vec<UserOpState>>();
        let flags = states.iter().map(UserOpFlag::from).collect::<Vec<_>>();
        flags.iter().enumerate().for_each(|(i, flag)| {
            if i < flags.len() - 1 {
                assert!(flag < &flags[i + 1])
            }
        })
    }

    #[test]
    fn flags_should_convert_to_state_correctly() {
        let states = UserOpState::iter().collect::<Vec<UserOpState>>();
        let flags = states.iter().map(UserOpFlag::from).collect::<Vec<_>>();
        let result = flags.iter().map(|f| f.into()).collect::<Vec<UserOpState>>();
        assert_eq!(result, states);
    }
}
