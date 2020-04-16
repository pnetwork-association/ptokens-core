use crate::{
    types::Result,
    traits::DatabaseInterface,
    check_debug_mode::check_debug_mode,
};

pub fn set_key_in_db_to_value<D>(
    db: D,
    key: String,
    value: String
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Setting key: {} in DB to value: {}", key, value);
    check_debug_mode()
        .and_then(|_| db.put(hex::decode(key)?, hex::decode(value)?, None))
        .map(|_| "{putting_value_in_database_suceeded:true}".to_string())
}

pub fn get_key_from_db<D>(
    db: D,
    key: String,
    data_sensitivity: Option<u8>,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Maybe getting key: {} from DB...", key);
    check_debug_mode()
        .and_then(|_| db.get(hex::decode(key.clone())?, data_sensitivity))
        .map(|value| format!("{{key:{},value:{}}}", key, hex::encode(value)))
}
