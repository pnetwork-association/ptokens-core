use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::ChallengesListEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengesList(Vec<ChallengesListEntry>);

impl fmt::Display for ChallengesList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", json!(self))
    }
}
