use std::fmt;

use common::{DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{Challenge, ChallengeStatus, Challenges, ChallengesError, ChallengesListEntry};
use crate::{db_utils::SentinelDbKeys, DbKey, DbUtilsT, SentinelDbUtils, SentinelError};

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize, Constructor, Deref, DerefMut)]
pub struct ChallengesList(Vec<ChallengesListEntry>);

impl fmt::Display for ChallengesList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", json!(self))
    }
}

impl ChallengesList {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get<D: DatabaseInterface>(db_utils: &SentinelDbUtils<D>) -> Self {
        if let Ok(x) = Self::get_from_db(db_utils, &SentinelDbKeys::get_challenges_list_db_key()) {
            x
        } else {
            warn!("no `ChallengesList` in db, defaulting to empty list");
            Self::default()
        }
    }

    fn entry_idx(&self, needle: &EthHash) -> Option<usize> {
        self.iter().position(|entry| entry.hash() == needle)
    }

    pub fn update_challenge_status<D: DatabaseInterface>(
        &mut self,
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
                let mut entry = self[idx].clone();
                let existing_status = entry.status();

                if existing_status < &status {
                    debug!("updating status from {existing_status} to {status}");
                    entry.status = status;
                    self[idx] = entry;
                    self.update_in_db(db_utils)?;
                    Ok(())
                } else {
                    warn!("cannot update status from {existing_status} to {status}");
                    Ok(())
                }
            },
        }
    }

    fn add_challenge<D: DatabaseInterface>(
        &mut self,
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
                self.push(entry);
                self.update_in_db(db_utils)
            },
        }
    }

    pub fn add_challenges<D: DatabaseInterface>(
        mut self,
        db_utils: &SentinelDbUtils<D>,
        challenges: Challenges,
    ) -> Result<(), SentinelError> {
        debug!("adding {} challenges to list...", challenges.len());
        challenges
            .iter()
            .cloned()
            .try_for_each(|c| self.add_challenge(db_utils, c))
    }

    pub fn remove_challenge<D: DatabaseInterface>(
        &mut self,
        db_utils: &SentinelDbUtils<D>,
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
                self.swap_remove(idx);
                self.update_in_db(db_utils)?;
                Ok(())
            },
        }
    }

    pub fn remove_challenges<D: DatabaseInterface>(
        &mut self,
        db_utils: &SentinelDbUtils<D>,
        hashes: Vec<EthHash>,
    ) -> Result<(), SentinelError> {
        hashes.iter().try_for_each(|h| self.remove_challenge(db_utils, h))
    }

    pub fn get_pending_challenges<D: DatabaseInterface>(
        &self,
        db_utils: SentinelDbUtils<D>,
    ) -> Result<Challenges, SentinelError> {
        Ok(Challenges::new(
            self.iter()
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

    pub fn get_challenge<D: DatabaseInterface>(
        &self,
        db_utils: &SentinelDbUtils<D>,
        hash: &EthHash,
    ) -> Result<Challenge, SentinelError> {
        match self.entry_idx(hash) {
            None => Err(ChallengesError::NotInList(*hash).into()),
            Some(_) => Challenge::from_bytes(
                &db_utils
                    .db()
                    .get(hash.as_bytes().to_vec(), MIN_DATA_SENSITIVITY_LEVEL)?,
            ),
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

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{challenges::test_utils::get_n_random_challenges, db_utils::DbUtilsT};

    #[test]
    fn should_get_empty_list_if_non_in_db() {
        let db = get_test_database();
        let db_utils = SentinelDbUtils::new(&db);
        let r = ChallengesList::get(&db_utils);
        let expected_r = ChallengesList::empty();
        assert_eq!(r, expected_r);
    }

    #[test]
    fn challenges_list_should_work_correctly() {
        let db = get_test_database();
        let db_utils = SentinelDbUtils::new(&db);
        let challenges = get_n_random_challenges(5);
        let list = ChallengesList::empty();
        list.add_challenges(&db_utils, challenges.clone()).unwrap();

        let mut expected_list = ChallengesList::new(
            challenges
                .iter()
                .map(|c| ChallengesListEntry::try_from(c).unwrap())
                .collect::<Vec<ChallengesListEntry>>(),
        );

        // NOTE: Check the list is saved correctly
        let mut list_from_db = ChallengesList::get(&db_utils);
        assert_eq!(list_from_db, expected_list);

        // NOTE: Now check each challenge is saved correctly.
        challenges.iter().for_each(|c| {
            let h = c.hash().unwrap();
            let c_from_db = Challenge::get_from_db(&db_utils, &h.into()).unwrap();
            assert_eq!(c, &c_from_db);
        });

        // NOTE: Test adding a challenge
        let new_challenge = get_n_random_challenges(1)[0].clone();
        let new_challenge_hash = new_challenge.hash().unwrap();
        list_from_db.add_challenge(&db_utils, new_challenge.clone()).unwrap();
        expected_list.push(ChallengesListEntry::try_from(new_challenge.clone()).unwrap());
        list_from_db = ChallengesList::get(&db_utils);
        assert_eq!(list_from_db, expected_list);
        assert_eq!(
            Challenge::get_from_db(&db_utils, &new_challenge_hash.into()).unwrap(),
            new_challenge
        );

        // Test removing a challenge
        let idx = 2;
        let challenge_to_remove = challenges[idx].clone();
        let challenge_to_remove_hash = challenge_to_remove.hash().unwrap();
        expected_list.swap_remove(idx);
        ChallengesList::get(&db_utils)
            .remove_challenge(&db_utils, &challenge_to_remove_hash)
            .unwrap();
        list_from_db = ChallengesList::get(&db_utils);
        assert_eq!(list_from_db, expected_list);
        assert!(Challenge::get_from_db(&db_utils, &challenge_to_remove_hash.into()).is_err());
    }
}
