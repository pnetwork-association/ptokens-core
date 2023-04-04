use common::Byte;
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

use super::UserOpState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Deref, Constructor, Serialize, Deserialize)]
pub struct UserOpFlags(Byte);

impl UserOpFlags {
    fn set_flag(&mut self, state: &UserOpState) {
        match state {
            UserOpState::Witnessed(..) => self.0 |= 0b0000_0001,
            UserOpState::Enqueued(..) => self.0 |= 0b0000_0010,
            UserOpState::Executed(..) => self.0 |= 0b0000_0100,
            UserOpState::Cancelled(..) => self.0 |= 0b0000_1000,
        }
    }

    fn is_set(&self, state: &UserOpState) -> bool {
        self.bit_is_set(state.to_bit_flag_idx())
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

    fn is_witnessed(&self) -> bool {
        self.bit_is_set(0)
    }

    fn is_enqueued(&self) -> bool {
        self.bit_is_set(1)
    }

    fn is_executed(&self) -> bool {
        self.bit_is_set(2)
    }

    fn is_cancelled(&self) -> bool {
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
    fn default_should_have_no_flags_set() {
        let user_op_flags = UserOpFlags::default();
        for n in 0..8 {
            assert!(!user_op_flags.bit_is_set(n))
        }
    }

    #[test]
    fn should_set_flags_for_each_state() {
        let states = UserOpState::iter().collect::<Vec<UserOpState>>();
        for state in states {
            let mut user_op_flags = UserOpFlags::default();
            assert!(!user_op_flags.is_set(&state));
            user_op_flags.set_flag(&state);
            assert!(user_op_flags.is_set(&state));
        }
    }

    #[test]
    fn should_not_overflow_when_shifting() {
        let user_op_flags = UserOpFlags::default();
        let n = 155;
        assert!(n > 8);
        let result = user_op_flags.bit_is_set(n);
        assert!(!result);
    }

    #[test]
    fn should_be_able_to_have_multiple_flags_set() {
        let mut user_op_flags = UserOpFlags::default();
        let side = BridgeSide::Native;
        let hash = EthHash::random();
        let s1 = UserOpState::Witnessed(side, hash);
        let s2 = UserOpState::Enqueued(side, hash);
        user_op_flags.set_flag(&s1);
        assert!(user_op_flags.is_set(&s1));
        user_op_flags.set_flag(&s2);
        assert!(user_op_flags.is_set(&s1));
        assert!(user_op_flags.is_set(&s2));
        assert!(user_op_flags.is_witnessed());
        assert!(user_op_flags.is_enqueued());
        assert!(!user_op_flags.is_executed());
        assert!(!user_op_flags.is_cancelled());
    }
}
