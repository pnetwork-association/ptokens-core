#![allow(unused)] // FIXME rm
                  //
use std::{cmp::PartialEq, fmt};

use common::{Byte, Bytes, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
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

#[derive(Clone, Debug, Default, Eq, Serialize, Deserialize, Constructor)]
pub struct UserOpListEntry {
    uid: EthHash,
    timestamp: u64,
    flag: UserOpFlag,
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
        Ok(Self::new(o.to_uid()?, get_utc_timestamp()?, o.to_flag()))
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

    fn get(&self, uid: &EthHash) -> Option<UserOpListEntry> {
        for entry in self.iter() {
            if &entry.uid == uid {
                return Some(entry.clone());
            }
        }
        None
    }

    fn process_op<D: DatabaseInterface>(db_utils: &SentinelDbUtils<D>, op: &UserOp) -> Result<(), UserOpError> {
        let uid = op.to_uid()?;
        let entry = UserOpListEntry::try_from(op)?;
        let flag = op.to_flag();
        let mut list = Self::get_from_db(db_utils, USER_OP_LIST.clone())?;

        match list.get(&uid) {
            Some(entry) => {
                debug!("user op found in db");
                if entry.flag.is_cancelled() {
                    warn!("user op found in db is cancelled - doing nothing");
                    Ok(())
                } else if entry.flag.is_executed() {
                    warn!("user op found in db is already exectuted - doing nothing");
                    Ok(())
                } else if entry.flag >= flag {
                    warn!("user op entry is <= that one in the database - doing nothing");
                    Ok(())
                } else if entry.flag < flag {
                    unimplemented!("TODO");
                    // if exist op is < this op, update existing one in the db, and update this list with new
                    // flag & timestamp etc
                    Ok(())
                } else {
                    Err(UserOpError::Process("Should never reach here!".into()))
                }
            },
            None => {
                debug!("Adding new user op to db");
                list.push(entry);
                op.put_in_db(db_utils)?;
                list.put_in_db(db_utils)?;
                Ok(())
            },
        }
    }

    fn process_ops<D: DatabaseInterface>(db_utils: &SentinelDbUtils<D>, ops: UserOps) -> Result<(), UserOpError> {
        ops.iter()
            .map(|op| Self::process_op(db_utils, op))
            .collect::<Result<Vec<_>, UserOpError>>()?;
        Ok(())
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
        let list_from_db = UserOpList::get_from_db(&db_utils, key).unwrap();
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
