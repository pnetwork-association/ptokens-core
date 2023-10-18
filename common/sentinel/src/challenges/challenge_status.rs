use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::ChallengesError;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ChallengeStatus {
    Null              = 0,
    Pending           = 1,
    Solved            = 2,
    Unsolved          = 3,
    PartiallyUnsolved = 4,
    Cancelled         = 5,
}

impl ChallengeStatus {
    pub(super) fn is_unsolved(&self) -> bool {
        matches!(self, Self::Pending | Self::Unsolved | Self::PartiallyUnsolved)
    }
}

impl Default for ChallengeStatus {
    fn default() -> Self {
        Self::Null
    }
}

impl TryFrom<u8> for ChallengeStatus {
    type Error = ChallengesError;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(Self::Null),
            1 => Ok(Self::Pending),
            2 => Ok(Self::Solved),
            3 => Ok(Self::Unsolved),
            4 => Ok(Self::PartiallyUnsolved),
            5 => Ok(Self::Cancelled),
            other => Err(Self::Error::CannotGetChallengeStatusFrom(format!("{other}"))),
        }
    }
}

impl From<ChallengeStatus> for u8 {
    fn from(s: ChallengeStatus) -> Self {
        match s {
            ChallengeStatus::Null => 0,
            ChallengeStatus::Pending => 1,
            ChallengeStatus::Solved => 2,
            ChallengeStatus::Unsolved => 3,
            ChallengeStatus::PartiallyUnsolved => 4,
            ChallengeStatus::Cancelled => 5,
        }
    }
}

impl FromStr for ChallengeStatus {
    type Err = ChallengesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "0" | "null" => Ok(Self::Null),
            "1" | "pending" => Ok(Self::Pending),
            "2" | "solved" => Ok(Self::Solved),
            "3" | "unsolved" => Ok(Self::Unsolved),
            "4" | "partiallyunsolved" => Ok(Self::PartiallyUnsolved),
            "5" | "cancelled" => Ok(Self::Cancelled),
            other => Err(Self::Err::CannotGetChallengeStatusFrom(other.to_string())),
        }
    }
}

impl fmt::Display for ChallengeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Null => "null",
            Self::Pending => "pending",
            Self::Solved => "solved",
            Self::Unsolved => "unsolved",
            Self::PartiallyUnsolved => "partiallyUnsolved",
            Self::Cancelled => "cancelled",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_u8_round_trip() {
        let n = 2u8;
        let status = ChallengeStatus::try_from(n).unwrap();
        let r: u8 = status.into();
        assert_eq!(n, r);
    }

    #[test]
    fn should_make_str_round_trip() {
        let s = "2";
        let status = ChallengeStatus::from_str(s).unwrap();
        let r = status.to_string();
        let expected_r = "solved";
        assert_eq!(r, expected_r);
    }
}
