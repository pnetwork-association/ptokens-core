use std::fmt;

use common::{get_prefixed_db_key, Byte, Bytes, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{SentinelError, UserOperations};

pub trait DbUtilsT {
    fn key(&self) -> DbKey;
    fn sensitivity_level(&self) -> Option<Byte>;
    fn bytes(&self) -> Result<Bytes, SentinelError>;
    fn from_bytes(bytes: &[Byte]) -> Result<Self, SentinelError>
    where
        Self: Sized;
}

#[derive(Clone, Debug, Eq, PartialEq, Deref, Serialize, Deserialize, Constructor)]
pub struct DbKey([Byte; 32]);

impl fmt::Display for DbKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl From<[u8; 32]> for DbKey {
    fn from(k: [u8; 32]) -> Self {
        Self(k)
    }
}

impl Into<Bytes> for DbKey {
    fn into(self) -> Bytes {
        self.to_vec()
    }
}

impl Into<Bytes> for &DbKey {
    fn into(self) -> Bytes {
        self.to_vec()
    }
}

macro_rules! create_db_keys {
    ($($name:ident),* $(,)?) => {
        lazy_static! {
            static ref SENTINEL_DB_KEYS: SentinelDbKeys = SentinelDbKeys::new($($name.clone())*);
            $(static ref $name: DbKey = get_prefixed_db_key(stringify!($name)).into();)*
        }

        paste! {
            impl<'a, D: DatabaseInterface> SentinelDbUtils<'a, D> {
                pub fn put<T: DbUtilsT>(&self, t: &T) -> Result<(), SentinelError> {
                    Ok(self
                        .db()
                        .put(
                            t.key().into(),
                            t.bytes()?,
                            t.sensitivity_level(),
                        )?
                    )
                }

                pub fn get_sensitive<T: DbUtilsT>(&self, key: &DbKey, sensitivity: Option<Byte>) -> Result<T, SentinelError> {
                    let bs = self.db().get(
                        key.into(),
                        if sensitivity.is_none() {
                            MIN_DATA_SENSITIVITY_LEVEL
                        } else {
                            sensitivity
                        },
                    )?;
                    T::from_bytes(&bs)
                }

                pub fn get<T: DbUtilsT>(&self, key: &DbKey) -> Result<T, SentinelError> {
                    self.get_sensitive(key, MIN_DATA_SENSITIVITY_LEVEL)
                }

                $(
                    fn [< get_ $name:lower _key >]() -> &'a DbKey {
                        &*$name
                    }
                )*
            }

            #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Constructor)]
            pub struct SentinelDbKeys {
                $([< $name:lower >]: DbKey,)*
            }

            impl fmt::Display for SentinelDbKeys {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    let s = json!({
                        $(stringify!($name): format!("0x{}", hex::encode(&self.[< $name:lower >].to_vec())),)*
                    }).to_string();

                    write!(f, "{s}")
                }
            }
        }
    }
}

create_db_keys!();

macro_rules! create_db_stuff {
    ($($name:ident),* $(,)?) => {
        lazy_static! {
            $(
                static ref $name: DbKey = get_prefixed_db_key(stringify!($name)).into();
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

                    pub fn [< get_ $name:lower >](&self) -> Result<UserOperations, SentinelError> {
                        let result = self.db().get(
                            Self::[< get_ $name:lower _key >](),
                            MIN_DATA_SENSITIVITY_LEVEL
                        );

                        match result {
                            Ok(bytes) => UserOperations::try_from(bytes),
                            Err(e) => {
                                warn!("Error getting {} from db, defaulting to empty set: {e}", stringify!([< $name:camel >]));
                                Ok(UserOperations::default())
                            },
                        }

                    }

                    pub fn [< put_ $name:lower >](&self, ops: UserOperations) -> Result<(), SentinelError> {
                        self.db().put(
                            Self::[< get_ $name:lower _key >](),
                            ops.try_into()?,
                            MIN_DATA_SENSITIVITY_LEVEL
                        )?;
                        Ok(())
                    }

                    pub fn [< replace_ $name:lower >](&self, ops: UserOperations) -> Result<(), SentinelError> {
                        let key = Self::[< get_ $name:lower _key >]();
                        self.db().delete(key.clone())?;
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
            use crate::UserOperation;
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
                        let expected_result = UserOperations::new(vec![x]);
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
                        assert_eq!(result, xs);
                        db_utils.[< replace_ $name:lower >](ys.clone()).unwrap();
                        result = db_utils.[< get_ $name:lower >]().unwrap();
                        assert_eq!(result, ys);
                    }
                )*
            }
        }
    }
}

create_db_stuff!(HOST_USER_OPERATIONS, NATIVE_USER_OPERATIONS);

pub struct SentinelDbUtils<'a, D: DatabaseInterface>(&'a D);

impl<'a, D: DatabaseInterface> SentinelDbUtils<'a, D> {
    pub fn new(db: &'a D) -> Self {
        Self(db)
    }
}
