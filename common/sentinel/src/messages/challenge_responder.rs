use std::fmt;

#[derive(Debug, Clone)]
pub enum ChallengeResponderMessages {
    RespondToChallenges,
    SetChallengeResponseFrequency(u64),
}

impl fmt::Display for ChallengeResponderMessages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::RespondToChallenges => "respond to pending challenges".to_string(),
            Self::SetChallengeResponseFrequency(n) => format!("set challenge response frequency to {n}"),
        };
        write!(f, "{s}")
    }
}
