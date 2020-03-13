#![cfg(test)]
use std::{
    sync::Mutex,
    collections::HashMap,
};
use crate::btc_on_eos::{
    errors::AppError,
    traits::DatabaseInterface,
    types::{
        Bytes,
        Result,
        DataSensitivity,
    },
};


#[cfg(test)]
pub const ROPSTEN_CONTRACT_ADDRESS: &str =
    "0x1Ee4D5f444d0Ab291D748049231dC9331b2f04C8";

#[cfg(test)]
pub const TEMPORARY_DATABASE_PATH: &str = "src/test_utils/temporary_database"; // FIXME RM!

pub fn get_sample_message_to_sign() -> &'static str {
    "Provable pToken!"
}

pub fn get_sample_message_to_sign_bytes() -> &'static [u8] {
    get_sample_message_to_sign()
        .as_bytes()
}

pub static DB_LOCK_ERRROR: &'static str = "✘ Cannot get lock on DB!";

pub struct TestDB(pub Mutex<HashMap<Bytes, Bytes>>);

impl TestDB {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

impl DatabaseInterface for TestDB {
    fn end_transaction(&self) -> Result<()> {
        Ok(())
    }

    fn start_transaction(&self) -> Result<()> {
        Ok(())
    }

    fn put(
        &self,
        key: Bytes,
        value: Bytes,
        _sensitivity: DataSensitivity,
    ) -> Result<()> {
        self
            .0
            .lock()
            .expect(DB_LOCK_ERRROR)
            .insert(key, value);
        Ok(())
    }

    fn delete(&self, key: Bytes) -> Result<()> {
        self
            .0
            .lock()
            .expect(DB_LOCK_ERRROR)
            .remove(&key);
        Ok(())
    }

    fn get(&self, key: Bytes, _sensitivity: DataSensitivity) -> Result<Bytes> {
        match self
            .0
            .lock()
            .expect(DB_LOCK_ERRROR)
            .get(&key) {
                Some(value) => Ok(value.to_vec()),
                None => Err(AppError::Custom(
                    "✘ Cannot find item in database!".to_string()
                ))
            }
    }
}

pub fn get_test_database() -> TestDB {
    TestDB::new()
}
