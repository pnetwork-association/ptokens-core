#![allow(unused)] // FIXME rm!

use common::{get_prefixed_db_key, Byte, Bytes, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};

use crate::{relevant_logs::RelevantLogs, SentinelError, UserOperations};

type DbKey = [Byte; 32];

macro_rules! create_db_keys {
    ($($name:ident),* $(,)?) => {
        lazy_static! {
            $(
                static ref $name: DbKey = get_prefixed_db_key(stringify!($name));
            )*
        }

        impl<'a, D: DatabaseInterface> SentinelDbUtils<'a, D> {
            fn db(&self) -> &D {
                self.0
            }

            paste! {
                $(
                    fn [< get_ $name:lower _key >]() -> Bytes {
                        $name.to_vec()
                    }

                    fn [< get_ $name:lower >](&self) -> Result<UserOperations, SentinelError> {
                        let result = self.db().get(
                            Self::[< get_ $name:lower _key >](),
                            MIN_DATA_SENSITIVITY_LEVEL
                        );

                        match result {
                            Ok(bytes) => UserOperations::try_from(bytes),
                            Err(e) => {
                                warn!("Error getting {} from db, defaulting to empty set!", stringify!([< $name:camel >]));
                                Ok(UserOperations::default())
                            },
                        }

                    }

                    fn [< put_ $name:lower >](&self, ops: UserOperations) -> Result<(), SentinelError> {
                        self.db().put(
                            Self::[< get_ $name:lower _key >](),
                            ops.try_into()?,
                            MIN_DATA_SENSITIVITY_LEVEL
                        )?;
                        Ok(())
                    }

                    fn [< replace_ $name:lower >](&self, ops: UserOperations) -> Result<(), SentinelError> {
                        let key = Self::[< get_ $name:lower _key >]();
                        self.db().delete(key.clone());
                        self.db().put(key, ops.try_into()?, MIN_DATA_SENSITIVITY_LEVEL)?;
                        Ok(())
                    }

                    pub fn [< add_ $name:lower >](&self, ops: UserOperations) -> Result<(), SentinelError> {
                        let mut ops_from_db = self.[< get_ $name:lower >]()?;
                        ops_from_db.add(ops);
                        self.[< put_ $name:lower >](ops_from_db)
                    }
                )*
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use std::time::Duration;
            use crate::{
                UserOperation,
                get_utc_timestamp,
            };
            use common::get_test_database;

            paste! {
                $(
                    #[test]
                    fn [< should_get_empty_ $name:lower >]() {
                        let db = get_test_database();
                        let db_utils = SentinelDbUtils::new(&db);
                        let result = db_utils.[< get_ $name:lower >]().unwrap();
                        let expected_result = UserOperations::default();
                        assert_eq!(result, expected_result);
                    }

                    #[test]
                    fn [< should_get_and_put_ $name:lower _in_db>]() {
                        let db = get_test_database();
                        let db_utils = SentinelDbUtils::new(&db);
                        let mut x = UserOperation::default();
                        x.set_destination_account("some account".into());
                        let mut expected_result = UserOperations::new(vec![x]);
                        db_utils.[< put_ $name:lower >](expected_result.clone()).unwrap();
                        let result = db_utils.[< get_ $name:lower >]().unwrap();
                        assert_eq!(result, expected_result);
                    }

                    #[test]
                    fn [< should_add_ $name:lower in_db>]() {
                        let db = get_test_database();
                        let db_utils = SentinelDbUtils::new(&db);
                        let mut x = UserOperation::default();
                        x.set_destination_account("some account".into());
                        let xs = UserOperations::new(vec![x.clone()]);
                        db_utils.[< put_ $name:lower >](xs.clone()).unwrap();
                        let mut result = db_utils.[< get_ $name:lower >]().unwrap();
                        assert_eq!(result, xs);
                        let mut y = UserOperation::default();
                        y.set_destination_account("some other account".into());
                        let ys = UserOperations::new(vec![y.clone()]);
                        assert_ne!(x, y);
                        let expected_result = UserOperations::new(vec![x, y]);
                        db_utils.[< add_ $name:lower >](ys).unwrap();
                        result = db_utils.[< get_ $name:lower >]().unwrap();
                        assert_eq!(result, expected_result);
                    }

                    #[test]
                    fn [< should_replace_ $name:lower _in_db >]() {
                        let db = get_test_database();
                        let db_utils = SentinelDbUtils::new(&db);
                        let mut x = UserOperation::default();
                        x.set_destination_account("some account".into());
                        let mut y = UserOperation::default();
                        y.set_destination_account("some other account".into());
                        assert_ne!(x, y);
                        let xs = UserOperations::new(vec![x.clone()]);
                        let ys = UserOperations::new(vec![y.clone()]);
                        assert_ne!(xs, ys);
                        db_utils.[< put_ $name:lower >](xs.clone()).unwrap();
                        let mut result = db_utils.[< get_ $name:lower >]().unwrap();
                        db_utils.[< replace_ $name:lower >](ys.clone()).unwrap();
                        result = db_utils.[< get_ $name:lower >]().unwrap();
                        assert_eq!(result, ys);
                    }
                )*
            }
        }
    }
}

create_db_keys!(HOST_USER_OPERATIONS, NATIVE_USER_OPERATIONS);

pub struct SentinelDbUtils<'a, D: DatabaseInterface>(&'a D);

impl<'a, D: DatabaseInterface> SentinelDbUtils<'a, D> {
    pub fn new(db: &'a D) -> Self {
        Self(db)
    }
}
