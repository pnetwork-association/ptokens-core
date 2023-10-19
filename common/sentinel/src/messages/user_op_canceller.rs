use std::fmt;

#[derive(Debug)]
pub enum UserOpCancellerMessages {
    CancelUserOps,
    SetFrequency(u64),
}

impl fmt::Display for UserOpCancellerMessages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::CancelUserOps => "cancel user ops".to_string(),
            Self::SetFrequency(n) => format!("set user op cancellation frequency to {n}"),
        };
        write!(f, "{s}")
    }
}
