use std::fmt;

use common::{get_prefixed_db_key, Byte, Bytes, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};
use derive_more::{Constructor, Deref};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::SentinelError;

pub trait DbUtilsT {
    fn key(&self) -> Result<DbKey, SentinelError>;
    fn sensitivity() -> Option<Byte>;
    fn from_bytes(bytes: &[Byte]) -> Result<Self, SentinelError>
    where
        Self: Sized;

    // TODO have Deserialize in the where clause so we can impl this here in the trait.
    fn bytes(&self) -> Result<Bytes, SentinelError>
    where
        Self: Serialize,
    {
        Ok(serde_json::to_vec(&self)?)
    }

    fn put_in_db<D: DatabaseInterface>(&self, db_utils: &SentinelDbUtils<D>) -> Result<(), SentinelError>
    where
        Self: Sized + Serialize,
    {
        let key = self.key()?;
        if Self::get_from_db(db_utils, &key).is_ok() {
            Err(SentinelError::KeyExists(key))
        } else {
            db_utils.put(self, &key)
        }
    }

    fn update_in_db<D: DatabaseInterface>(&self, db_utils: &SentinelDbUtils<D>) -> Result<(), SentinelError>
    where
        Self: Sized + Serialize,
    {
        db_utils.put(self, &self.key()?)
    }

    fn get_from_db<D: DatabaseInterface>(db_utils: &SentinelDbUtils<D>, key: &DbKey) -> Result<Self, SentinelError>
    where
        Self: Sized,
    {
        db_utils.get_sensitive(key, Self::sensitivity())
    }

    fn delete<D: DatabaseInterface>(&self, db_utils: &SentinelDbUtils<D>) -> Result<(), SentinelError>
    where
        Self: Sized,
    {
        Ok(db_utils.db().delete(self.key()?.to_vec())?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deref, Serialize, Deserialize, Constructor)]
pub struct DbKey([Byte; 32]);

impl fmt::Display for DbKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl From<[u8; 32]> for DbKey {
    fn from(k: [u8; 32]) -> Self {
        Self(k)
    }
}

impl From<EthHash> for DbKey {
    fn from(h: EthHash) -> Self {
        Self(h.0)
    }
}

impl From<&EthHash> for DbKey {
    fn from(h: &EthHash) -> Self {
        Self(h.0)
    }
}

impl From<DbKey> for Bytes {
    fn from(val: DbKey) -> Self {
        val.to_vec()
    }
}

impl From<&DbKey> for Bytes {
    fn from(val: &DbKey) -> Self {
        val.to_vec()
    }
}

macro_rules! create_db_keys {
    ($($name:ident),* $(,)?) => {
        paste! {
            lazy_static! {
                pub(crate) static ref SENTINEL_DB_KEYS: SentinelDbKeys = SentinelDbKeys::new($($name.clone(),)*);
                pub(crate) $(static ref $name: DbKey = get_prefixed_db_key(stringify!($name)).into();)*
            }

            impl<'a, D: DatabaseInterface> SentinelDbUtils<'a, D> {
                pub(crate) fn db(&self) -> &D {
                    self.0
                }

                pub fn put<T: DbUtilsT + Serialize>(&self, t: &T, key: &DbKey) -> Result<(), SentinelError> {
                    Ok(self
                        .db()
                        .put(
                            key.into(),
                            t.bytes()?,
                            T::sensitivity(),
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

                pub fn key_exists<T: DbUtilsT>(&self, key: &DbKey) -> bool {
                    let r: Result<_, _> = self.db().get(key.into(), MIN_DATA_SENSITIVITY_LEVEL);
                    r.is_ok()
                }

                $(
                    #[allow(unused)] // NOTE: Not all key getters are used.
                    pub fn [< get_ $name:lower _key >]() -> &'a DbKey {
                        &*$name
                    }
                )*
            }

            #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
            pub struct SentinelDbKeys {
                $([< $name:lower >]: DbKey,)*
            }

            impl SentinelDbKeys {
                pub fn new($([< $name:lower >]: DbKey,)*) -> Self {
                    Self { $([< $name:lower >]),* }
                }

                $(
                    #[allow(unused)] // NOTE: Not all key getters are used.
                    pub fn [< get_ $name:lower _db_key >]() -> DbKey {
                        $name.clone()
                    }
                )*

            }

            impl fmt::Display for SentinelDbKeys {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    let j = json!({
                        $(stringify!($name): format!("0x{}", hex::encode(&self.[< $name:lower >].to_vec())),)*
                    });
                    match serde_json::to_string_pretty(&j) {
                        Ok(s) => write!(f, "{s}"),
                        Err(e) => write!(f, "{e}"),
                    }
                }
            }
        }
    }
}

create_db_keys!(USER_OP_LIST, ACTOR_INCLUSION_PROOF, CHALLENGES_LIST);

pub struct SentinelDbUtils<'a, D: DatabaseInterface>(&'a D);

impl<'a, D: DatabaseInterface> SentinelDbUtils<'a, D> {
    pub fn new(db: &'a D) -> Self {
        Self(db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_sentinel_db_keys_json() {
        println!("{}", *SENTINEL_DB_KEYS);
    }
}
