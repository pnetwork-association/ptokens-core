use common_rocksdb::get_db;
use lib::{check_init, CoreState, SentinelConfig, SentinelError};
use serde_json::json;

pub fn get_core_state(config: &SentinelConfig) -> Result<String, SentinelError> {
    let db = get_db()?;
    check_init(&db)?;
    Ok(json!({"jsonrpc": "2.0", "result": CoreState::get(&db, &config.core_config.core_type)?}).to_string())
}
