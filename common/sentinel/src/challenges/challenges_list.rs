use std::fmt;

use common::{DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Challenge, ChallengeStatus, Challenges, ChallengesListEntry};
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

    fn entry_idx(&self, needle: &EthHash) -> Option<usize> {
        self.0.iter().position(|entry| entry.hash() == needle)
    }

    pub fn update_challenge_status<D: DatabaseInterface>(
        mut self,
        db_utils: &SentinelDbUtils<D>,
        hash: &EthHash,
        status: ChallengeStatus,
    ) -> Result<(), SentinelError> {
        debug!("updating status in challenges list to {status}");
        match self.entry_idx(hash) {
            None => {
                warn!("no challenge entry with hash {hash} in list");
                Ok(())
            },
            Some(idx) => {
                let mut entry = self.0[idx].clone();
                let existing_status = entry.status();

                if existing_status < &status {
                    debug!("updating status from {existing_status} to {status}");
                    entry.status = status;
                    self.0[idx] = entry;
                    self.update_in_db(db_utils)?;
                    Ok(())
                } else {
                    warn!("cannot update status from {existing_status} to {status}");
                    Ok(())
                }
            },
        }
    }

    pub fn add_challenge<D: DatabaseInterface>(
        mut self,
        db_utils: &SentinelDbUtils<D>,
        challenge: Challenge,
    ) -> Result<(), SentinelError> {
        let hash = challenge.hash()?;

        match self.entry_idx(&hash) {
            Some(_) => {
                warn!("not adding challenge to list, it already exists!");
                Ok(())
            },
            None => {
                debug!("adding challenge to challenges list");
                challenge.put_in_db(db_utils)?;
                let entry = ChallengesListEntry::try_from(challenge)?;
                self.0.push(entry);
                self.put_in_db(db_utils)
            },
        }
    }

    pub fn remove_challenge<D: DatabaseInterface>(
        mut self,
        db_utils: SentinelDbUtils<D>,
        hash: &EthHash,
    ) -> Result<(), SentinelError> {
        match self.entry_idx(hash) {
            None => {
                warn!("not challenge with hash {hash} in challenges list, not removing anything");
                Ok(())
            },
            Some(idx) => {
                debug!("removing challenge with hash {hash} from list");
                db_utils.db().delete(hash.as_bytes().to_vec())?; // NOTE: Delete the actual challenge from the db
                self.0.swap_remove(idx);
                self.put_in_db(&db_utils)?;
                Ok(())
            },
        }
    }

    pub fn get_pending_challenges<D: DatabaseInterface>(
        &self,
        db_utils: SentinelDbUtils<D>,
    ) -> Result<Challenges, SentinelError> {
        Ok(Challenges::new(
            self.0
                .iter()
                .filter(|entry| entry.status == ChallengeStatus::Pending)
                .map(|entry| {
                    Challenge::from_bytes(
                        &db_utils
                            .db()
                            .get(entry.hash().as_bytes().to_vec(), MIN_DATA_SENSITIVITY_LEVEL)?,
                    )
                })
                .collect::<Result<Vec<Challenge>, SentinelError>>()?,
        ))
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
