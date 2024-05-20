use common::{AppError as CommonError, Bytes, DatabaseInterface};

use crate::android::{type_aliases::DataSensitivity, Database};

impl DatabaseInterface for Database<'_> {
    fn end_transaction(&self) -> Result<(), CommonError> {
        self.end_transaction().map_err(|e| e.into())
    }

    fn start_transaction(&self) -> Result<(), CommonError> {
        self.start_transaction().map_err(|e| e.into())
    }

    fn delete(&self, key: Bytes) -> Result<(), CommonError> {
        self.delete(&key).map_err(|e| e.into())
    }

    fn get(&self, key: Bytes, data_sensitivity: DataSensitivity) -> Result<Bytes, CommonError> {
        self.get(&key, data_sensitivity).map_err(|e| e.into())
    }

    fn put(&self, key: Bytes, value: Bytes, data_sensitivity: DataSensitivity) -> Result<(), CommonError> {
        self.put(&key, &value, data_sensitivity).map_err(|e| e.into())
    }
}
