use std::fmt;

use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Challenge, ChallengeState, ChallengesError};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Getters, Constructor)]
pub struct ChallengesListEntry {
    hash: EthHash,
    pub status: ChallengeState,
}

impl TryFrom<Challenge> for ChallengesListEntry {
    type Error = ChallengesError;

    fn try_from(c: Challenge) -> Result<Self, Self::Error> {
        Self::try_from(&c)
    }
}

impl TryFrom<&Challenge> for ChallengesListEntry {
    type Error = ChallengesError;

    fn try_from(c: &Challenge) -> Result<Self, Self::Error> {
        // NOTE: If we're constructing a new list entry from a challenge, we assume that challenge
        // is pending.
        Ok(Self::new(c.hash()?, ChallengeState::Pending))
    }
}

impl fmt::Display for ChallengesListEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let j = json!({
            "challengeStatus": self.status,
            "challengeHash": format!("0x{}", hex::encode(self.hash.as_bytes())),
        });
        write!(f, "{j}")
    }
}
