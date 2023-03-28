use std::{cell::RefCell, collections::HashMap};

use common::{AppError as PTokensCoreError, Bytes, DatabaseInterface, Result as PTokensResult};
use rocksdb::{WriteBatch, DB};

use crate::RocksdbDatabaseError;

pub const DATABASE_PATH: &str = "./database";

type DataSensitivity = Option<u8>;

pub struct Database {
    pub rocks_db: rocksdb::DB,
    pub batch_db_ops: RefCell<Vec<DbOp>>,
    pub keys_to_delete: RefCell<Vec<Bytes>>,
    pub hashmap: RefCell<HashMap<Bytes, Bytes>>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            batch_db_ops: RefCell::new(vec![]),
            keys_to_delete: RefCell::new(vec![]),
            hashmap: RefCell::new(HashMap::new()),
            rocks_db: DB::open_default(DATABASE_PATH).expect("Cannot create default rocksdb instance!"),
        }
    }
}

pub enum DbOp {
    Delete(Bytes),
    Put(Bytes, Bytes),
}

impl Database {
    fn open_inner(path: &str) -> Result<Self, RocksdbDatabaseError> {
        Ok(Self {
            rocks_db: DB::open_default(path)?,
            hashmap: RefCell::new(HashMap::new()),
            batch_db_ops: RefCell::new(Vec::new()),
            keys_to_delete: RefCell::new(Vec::new()),
        })
    }

    pub fn open() -> Result<Self, RocksdbDatabaseError> {
        Self::open_inner(DATABASE_PATH)
    }

    pub fn open_at_path(path: &str) -> Result<Self, RocksdbDatabaseError> {
        Self::open_inner(path)
    }
}

impl DatabaseInterface for Database {
    fn end_transaction(&self) -> PTokensResult<()> {
        info!("✔ Ending DB transaction in app...");
        let mut batch = WriteBatch::default();
        self.batch_db_ops
            .borrow()
            .iter()
            .map(|db_op| match db_op {
                DbOp::Delete(key) => batch.delete(key),
                DbOp::Put(key, value) => batch.put(key, value),
            })
            .for_each(drop);
        trace!("✔ Batch writing to DB...");
        match self.rocks_db.write(batch) {
            Ok(_) => {
                trace!("✔ Atomic batch write successful!");
                Ok(())
            },
            Err(e) => {
                trace!("✘ Error batch writing to DB: {}", &e);
                Err(PTokensCoreError::Custom(e.to_string()))
            },
        }
    }

    fn start_transaction(&self) -> PTokensResult<()> {
        info!("✔ Starting DB transaction in app...");
        Ok(())
    }

    fn put(&self, key: Bytes, value: Bytes, _sensitivity: DataSensitivity) -> PTokensResult<()> {
        trace!("✔ Putting key in hashmap...");
        self.hashmap.borrow_mut().insert(key.clone(), value.clone());
        trace!("✔ Checking if key is in delete list... ");
        match self.keys_to_delete.borrow().contains(&key) {
            true => {
                trace!("✔ Removing key from delete list... ");
                self.hashmap.borrow_mut().remove(&key);
            },
            false => {
                trace!("✔ Key not in delete list, nothing to remove.");
            },
        };
        self.batch_db_ops.borrow_mut().push(DbOp::Put(key, value));
        Ok(())
    }

    fn delete(&self, key: Bytes) -> PTokensResult<()> {
        trace!("✔ Removing key from hashmap...");
        self.hashmap.borrow_mut().remove(&key);
        trace!("✔ Adding key to `to_delete` list...");
        self.keys_to_delete.borrow_mut().push(key.clone());
        self.batch_db_ops.borrow_mut().push(DbOp::Delete(key));
        Ok(())
    }

    fn get(&self, key: Bytes, _sensitivity: DataSensitivity) -> PTokensResult<Bytes> {
        trace!("✔ Getting key: {}", hex::encode(&key));
        let not_in_db_error = "Cannot find item in database!".to_string();
        if self.keys_to_delete.borrow().contains(&key) {
            trace!("✔ Key already in delete list ∴ 'not found'!");
            Err(PTokensCoreError::Custom(not_in_db_error))
        } else {
            trace!("✔ Checking hashmap for key...");
            match self.hashmap.borrow().get(&key) {
                Some(value) => {
                    trace!("✔ Key found in hashmap!");
                    Ok(value.to_vec())
                },
                None => {
                    trace!("✘ Key NOT in hashmap!");
                    trace!("✔ Looking in underlying DB...");
                    match self.rocks_db.get(key) {
                        Ok(Some(value)) => {
                            trace!("✔ Key found in DB!");
                            Ok(value.to_vec())
                        },
                        Err(e) => Err(PTokensCoreError::Custom(e.to_string())),
                        Ok(None) => Err(PTokensCoreError::Custom(not_in_db_error)),
                    }
                },
            }
        }
    }
}
