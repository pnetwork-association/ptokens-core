use std::str::FromStr;

use ethereum_types::U256;
use serde::{Deserialize, Serialize};

use super::ActorsError;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActorType {
    Governance = 0,
    Guardian   = 1,
    Sentinel   = 2,
}

impl ActorType {
    #[cfg(test)]
    pub(crate) fn random() -> Self {
        use rand::Rng;
        Self::try_from(rand::thread_rng().gen_range(0..3)).unwrap()
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Governance => &[0],
            Self::Guardian => &[1],
            Self::Sentinel => &[2],
        }
    }
}

impl From<ActorType> for u8 {
    fn from(val: ActorType) -> Self {
        match val {
            ActorType::Governance => 0,
            ActorType::Guardian => 1,
            ActorType::Sentinel => 2,
        }
    }
}

impl From<ActorType> for U256 {
    fn from(val: ActorType) -> Self {
        let x: u8 = val.into();
        U256::from(x)
    }
}

impl TryFrom<u8> for ActorType {
    type Error = ActorsError;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        Self::from_str(&format!("{n}"))
    }
}

impl From<&ActorType> for u8 {
    fn from(t: &ActorType) -> u8 {
        match t {
            ActorType::Governance => 0,
            ActorType::Guardian => 1,
            ActorType::Sentinel => 2,
        }
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
