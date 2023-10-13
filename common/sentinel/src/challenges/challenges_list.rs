use std::fmt;

use common::{DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::ChallengesListEntry;
use crate::{db_utils::SentinelDbKeys, DbKey, DbUtilsT, SentinelDbUtils, SentinelError};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChallengesList(Vec<ChallengesListEntry>);

impl fmt::Display for ChallengesList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", json!(self))
    }
}

impl ChallengesList {
    pub fn get<D: DatabaseInterface>(db_utils: &SentinelDbUtils<D>) -> Self {
        if let Ok(x) = Self::get_from_db(db_utils, &SentinelDbKeys::get_challenges_list_db_key()) {
            x
        } else {
            warn!("no `ChallengesList` in db, defaulting to empty list");
            Self::default()
        }
    }
}

impl DbUtilsT for ChallengesList {
    fn key(&self) -> Result<DbKey, SentinelError> {
        Ok(SentinelDbKeys::get_challenges_list_db_key())
    }

    fn sensitivity() -> Option<u8> {
        MIN_DATA_SENSITIVITY_LEVEL
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, SentinelError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
