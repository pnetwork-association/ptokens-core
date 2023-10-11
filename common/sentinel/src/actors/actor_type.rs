use std::str::FromStr;

use ethereum_types::U256;
use serde::{Deserialize, Serialize};

use super::ActorsError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorType {
    Governance = 0,
    Guardian   = 1,
    Sentinel   = 2,
}

impl ActorType {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Governance => &[0],
            Self::Guardian => &[1],
            Self::Sentinel => &[2],
        }
    }
}

impl TryFrom<u8> for ActorType {
    type Error = ActorsError;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        Self::from_str(&format!("{n}"))
    }
}

impl FromStr for ActorType {
    type Err = ActorsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "2" | "sentinel" => Ok(Self::Sentinel),
            "1" | "guardian" => Ok(Self::Guardian),
            "0" | "governance" => Ok(Self::Governance),
            other => Err(ActorsError::CannotDetermineActorType(other.to_string())),
        }
    }
}

impl TryFrom<&U256> for ActorType {
    type Error = ActorsError;

    fn try_from(u: &U256) -> Result<Self, Self::Error> {
        match u.as_u64() {
            0 => Ok(Self::Governance),
            1 => Ok(Self::Guardian),
            2 => Ok(Self::Sentinel),
            n => Err(ActorsError::CannotGetActorType(n)),
        }
    }
}

impl Default for ActorType {
    fn default() -> Self {
        Self::Governance
    }
}
