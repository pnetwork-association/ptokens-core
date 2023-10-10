use ethereum_types::U256;
use serde::{Deserialize, Serialize};

use super::ActorsError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorType {
    Governance = 0,
    Guardian   = 1,
    Sentinel   = 2,
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
