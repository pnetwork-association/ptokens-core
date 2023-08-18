use std::{cmp::PartialEq, fmt};

use common::{Byte, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{UserOp, UserOpError, UserOpFlag, UserOps};
use crate::{
    db_utils::{DbKey, DbUtilsT, USER_OP_LIST},
    get_utc_timestamp,
    SentinelDbUtils,
    SentinelError,
};

#[derive(Clone, Copy, Debug, Default, Eq, Serialize, Deserialize, Constructor)]
pub struct UserOpListEntry {
    uid: EthHash,
    timestamp: u64,
    flag: UserOpFlag,
}

impl UserOpListEntry {
    pub(super) fn uid(&self) -> EthHash {
        self.uid
    }

    fn set_flag(mut self, flag: UserOpFlag) {
        debug!("setting flag in user op list entry from {} to {flag}", self.flag);
        self.flag = flag;
    }
}

impl PartialEq for UserOpListEntry {
    fn eq(&self, other: &Self) -> bool {
        // NOTE: We only are about the uid when testing for equality!
        self.uid == other.uid
    }
}

impl TryFrom<&UserOp> for UserOpListEntry {
    type Error = UserOpError;

    fn try_from(o: &UserOp) -> Result<Self, Self::Error> {
        Ok(Self::new(o.uid()?, get_utc_timestamp()?, o.to_flag()))
    }
}

impl fmt::Display for UserOpListEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match serde_json::to_string_pretty(self) {
            Ok(s) => s,
            Err(e) => format!("could not fmt `UserOpListEntry` {e}"),
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize, Deref, DerefMut, Constructor)]
pub struct UserOpList(Vec<UserOpListEntry>);

impl fmt::Display for UserOpList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match serde_json::to_string_pretty(self) {
            Ok(s) => s,
            Err(e) => format!("could not fmt `UserOpList` {e}"),
        };
        write!(f, "{s}")
    }
}

impl DbUtilsT for UserOpList {
    fn key(&self) -> Result<DbKey, SentinelError> {
        Ok(USER_OP_LIST.clone())
    }

    fn sensitivity() -> Option<Byte> {
        MIN_DATA_SENSITIVITY_LEVEL
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self, SentinelError> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

impl UserOpList {
    fn includes(&self, uid: &EthHash) -> bool {
        for entry in self.iter() {
            if &entry.uid == uid {
                return true;
            }
        }
        false
    }

    fn get_entry(&self, uid: &EthHash) -> Option<UserOpListEntry> {
        for entry in self.iter() {
            if &entry.uid == uid {
                return Some(*entry);
            }
        }
        None
    }

    fn upsert(&mut self, entry: UserOpListEntry) -> Result<(), UserOpError> {
        if self.includes(&entry.uid()) {
            debug!("updating entry in `UserOpList` to : {entry}");
            let idx = self.iter().position(|e| e == &entry).expect("this should exist");
            self[idx] = entry;
        } else {
            debug!("adding entry to `UserOpList`: {entry}");
            self.push(entry);
        };
        Ok(())
    }

    pub fn remove_entry<D: DatabaseInterface>(
        &mut self,
        db_utils: &SentinelDbUtils<D>,
        uid: &EthHash,
    ) -> Result<bool, UserOpError> {
        if !self.includes(uid) {
            debug!("no entry with uid {uid} doing nothing");
            Ok(false)
        } else {
            let idx = self
                .iter()
                .position(|entry| &entry.uid == uid)
                .expect("this should exist");
            let entry = self[idx];
            let op = UserOp::get_from_db(db_utils, &entry.uid.into())?;
            debug!("removing entry from list {entry} @ idx {idx}");
            op.delete(db_utils)?;
            self.remove(idx);
            self.update_in_db(db_utils)?;
            Ok(true)
        }
    }

    fn handle_is_not_in_list<D: DatabaseInterface>(
        &mut self,
        db_utils: &SentinelDbUtils<D>,
        op: UserOp,
    ) -> Result<(), UserOpError> {
        debug!("adding user op to db: {op}");
        self.push(UserOpListEntry::try_from(&op)?);
        op.put_in_db(db_utils)?;
        self.update_in_db(db_utils)?;
        Ok(())
    }

    fn handle_is_in_list<D: DatabaseInterface>(
        &mut self,
        db_utils: &SentinelDbUtils<D>,
        op: UserOp,
        list_entry: UserOpListEntry,
    ) -> Result<(), UserOpError> {
        debug!("user op found in db");
        let mut op_from_db = UserOp::get_from_db(db_utils, &op.key()?)?;

        op_from_db.maybe_update_state(op)?;
        op_from_db.update_in_db(db_utils)?;

        // NOTE: We can safely call this with no checks since the above state will only have
        // changed if it's more advanced.
        list_entry.set_flag(op_from_db.to_flag());
        self.upsert(list_entry)?;
        self.update_in_db(db_utils)?;

        Ok(())
    }

    fn process_op<D: DatabaseInterface>(
        &mut self,
        op: UserOp,
        db_utils: &SentinelDbUtils<D>,
    ) -> Result<(), UserOpError> {
        if let Some(entry) = self.get_entry(&op.uid()?) {
            self.handle_is_in_list(db_utils, op, entry)
        } else {
            self.handle_is_not_in_list(db_utils, op)
        }
    }

    pub fn process_ops<D: DatabaseInterface>(
        &mut self,
        ops: UserOps,
        db_utils: &SentinelDbUtils<D>,
    ) -> Result<(), SentinelError> {
        ops.iter()
            .map(|op| self.process_op(op.clone(), db_utils))
            .collect::<Result<Vec<()>, UserOpError>>()?;
        Ok(())
    }

    pub fn get<D: DatabaseInterface>(db_utils: &SentinelDbUtils<D>) -> Self {
        Self::get_from_db(db_utils, &USER_OP_LIST).unwrap_or_default()
    }

    pub fn user_ops<D: DatabaseInterface>(db_utils: &SentinelDbUtils<D>) -> Result<UserOps, SentinelError> {
        let list = Self::get(db_utils);
        let ops = list
            .iter()
            .map(|entry| UserOp::get_from_db(db_utils, &entry.uid().into()))
            .collect::<Result<Vec<UserOp>, SentinelError>>()?;
        Ok(UserOps::new(ops))
    }
}

#[cfg(test)]
mod tests {
    use common::get_test_database;

    use super::*;
    use crate::SentinelDbUtils;

    #[test]
    fn should_out_and_get_user_op_list_in_db() {
        let db = get_test_database();
        let db_utils = SentinelDbUtils::new(&db);
        let mut user_op = UserOp::default();
        user_op.set_destination_account("some account".into());
        assert_ne!(user_op, UserOp::default());
        let list_entry = UserOpListEntry::try_from(&user_op).unwrap();
        let list = UserOpList::new(vec![list_entry]);
        list.put_in_db(&db_utils).unwrap();
        let key = UserOpList::default().key().unwrap();
        let list_from_db = UserOpList::get_from_db(&db_utils, &key).unwrap();
        assert_eq!(list_from_db, list);
    }

    #[test]
    fn should_be_equal_if_uid_equal_but_not_flags() {
        let mut op_1 = UserOpListEntry::default();
        let mut op_2 = UserOpListEntry::default();
        let flag_1 = UserOpFlag::new(42);
        let flag_2 = UserOpFlag::new(24);
        assert_ne!(flag_1, flag_2);
        assert_eq!(op_1, op_2);
        op_1.flag = flag_1;
        op_2.flag = flag_2;
        assert_eq!(op_1, op_2);
    }
}
