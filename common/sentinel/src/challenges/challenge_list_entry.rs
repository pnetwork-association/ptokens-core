use std::fmt;

use derive_getters::Getters;
use derive_more::Constructor;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Challenge, ChallengeStatus, ChallengesError};

#[derive(Clone, Debug, Serialize, Deserialize, Getters, Constructor)]
pub(super) struct ChallengesListEntry {
    hash: EthHash,
    status: ChallengeStatus,
}

impl TryFrom<Challenge> for ChallengesListEntry {
    type Error = ChallengesError;

    fn try_from(c: Challenge) -> Result<Self, Self::Error> {
        // NOTE: If we're constructing a new list entry from a challenge, we assume that challenge
        // to be pending.
        Ok(Self::new(c.hash()?, ChallengeStatus::Pending))
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
