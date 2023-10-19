use std::{fmt, str::FromStr};

use common::Bytes;
use common_eth::encode_fxn_call;
use serde::{Deserialize, Serialize};

use super::{Challenge, ChallengesError};

const GET_CHALLENGE_STATUS_ABI: &str = "[{\"inputs\":[{\"components\":[{\"internalType\":\"uint256\",\"name\":\"nonce\",\"type\":\"uint256\"},{\"internalType\":\"address\",\"name\":\"actor\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"challenger\",\"type\":\"address\"},{\"internalType\":\"uint64\",\"name\":\"timestamp\",\"type\":\"uint64\"},{\"internalType\":\"bytes4\",\"name\":\"networkId\",\"type\":\"bytes4\"}],\"internalType\":\"struct IPNetworkHub.Challenge\",\"name\":\"challenge\",\"type\":\"tuple\"}],\"name\":\"getChallengeStatus\",\"outputs\":[{\"internalType\":\"enum IPNetworkHub.ChallengeStatus\",\"name\":\"\",\"type\":\"uint8\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]";

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

    pub fn encode_rpc_call_data(challenge: &Challenge) -> Result<Bytes, ChallengesError> {
        let encoded = encode_fxn_call(GET_CHALLENGE_STATUS_ABI, "getChallengeStatus", &[
            challenge.to_eth_abi_token()?
        ])?;
        Ok(encoded)
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

impl TryFrom<Bytes> for ChallengeStatus {
    type Error = ChallengesError;

    fn try_from(bs: Bytes) -> Result<Self, Self::Error> {
        let name = "ChallengeStatus";
        debug!("getting '{name}' from bytes...");
        if bs.is_empty() {
            Err(ChallengesError::NotEnoughBytes {
                got: 0,
                expected: "1".to_string(),
                location: name.to_string(),
            })
        } else {
            Self::try_from(bs[0])
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
