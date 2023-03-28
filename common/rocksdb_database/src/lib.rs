#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate log;

mod errors;
mod rocks_db;

use crate::rocks_db::Database;

pub fn get_db() -> Result<Database, RocksdbDatabaseError> {
    Database::open()
}

pub fn get_db_at_path(p: &str) -> Result<Database, RocksdbDatabaseError> {
    Database::open_at_path(p)
}

pub use crate::errors::RocksdbDatabaseError;
