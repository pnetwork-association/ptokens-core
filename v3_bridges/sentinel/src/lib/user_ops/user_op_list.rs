use std::{cmp::PartialEq, fmt};

use common::{Byte, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{UserOp, UserOpError, UserOpFlag, UserOpState, UserOps};
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
        Ok(Self::new(o.uid(), get_utc_timestamp()?, o.to_flag()))
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

    fn handle_is_not_in_list<D: DatabaseInterface>(
        db_utils: &SentinelDbUtils<D>,
        op: UserOp,
        mut list: Self,
    ) -> Result<Option<UserOp>, UserOpError> {
        debug!("adding user op to db: {op}");
        list.push(UserOpListEntry::try_from(&op)?);
        op.put_in_db(db_utils)?;
        list.put_in_db(db_utils)?;
        match op.state() {
            UserOpState::Enqueued(..) => {
                // NOTE: We return this because it'll require a cancellation signature
                warn!("enqueued event found but not witnessed!");
                Ok(Some(op))
            },
            _ => Ok(None),
        }
    }

    fn handle_is_in_list<D: DatabaseInterface>(
        db_utils: &SentinelDbUtils<D>,
        op: UserOp,
        _list: Self,
        list_entry_from_db: UserOpListEntry,
    ) -> Result<(), UserOpError> {
        debug!("user op found in db");
        let user_op_state = op.state();
        let db_user_op_state: UserOpState = list_entry_from_db.flag.into();

        match (db_user_op_state, user_op_state) {
            (a, b) if a >= b => {
                warn!("user op already in db in same or more advanced state - doing nothing");
                Ok(())
            },
            _ => {
                debug!("updating user op in db to {op}");
                let mut op_from_db = UserOp::get_from_db(db_utils, &op.key()?)?;
                op_from_db.update_state(op)?;
                op_from_db.update_in_db(db_utils)?;
                Ok(())
            },
        }
    }

    fn process_op<D: DatabaseInterface>(
        db_utils: &SentinelDbUtils<D>,
        op: UserOp,
    ) -> Result<Option<UserOp>, UserOpError> {
        let list = Self::get_from_db(db_utils, &USER_OP_LIST)?;
        if let Some(entry) = list.get(&op.uid()) {
            Self::handle_is_in_list(db_utils, op, list, entry)?;
            Ok(None)
        } else {
            Self::handle_is_not_in_list(db_utils, op, list)
        }
    }

    pub fn process_ops<D: DatabaseInterface>(
        db_utils: &SentinelDbUtils<D>,
        ops: UserOps,
    ) -> Result<UserOps, SentinelError> {
        // FIXME get the list once!
        let mut ops_to_cancel = vec![];
        for op in ops.iter().cloned() {
            if let Some(returned_op) = Self::process_op(db_utils, op)? {
                ops_to_cancel.push(returned_op)
            }
        }
        Ok(UserOps::new(ops_to_cancel))
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
