use std::result::Result;

use common::{CoreType, DatabaseInterface};

use crate::SentinelError;

pub fn check_init<D: DatabaseInterface>(db: &D) -> Result<(), SentinelError> {
    info!("Checking core is initialized...");
    let host_is_initted = CoreType::host_core_is_initialized(db);
    let native_is_initted = CoreType::native_core_is_initialized(db);
    let err = "core is not initialized";
    if !host_is_initted {
        Err(SentinelError::Custom(format!("host {err}")))
    } else if !native_is_initted {
        Err(SentinelError::Custom(format!("native {err}")))
    } else {
        info!("Core is intialized!");
        Ok(())
    }
}
