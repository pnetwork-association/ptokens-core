use common_rocksdb_database::get_db_at_path;
use common_sentinel::{check_init, SentinelConfig, SentinelDbUtils, SentinelError, UserOpList};
use serde_json::json;

pub fn get_user_ops(config: &SentinelConfig) -> Result<String, SentinelError> {
    let db = get_db_at_path(&config.get_db_path())?;
    check_init(&db)?;
    let ops = UserOpList::user_ops(&SentinelDbUtils::new(&db))?;
    Ok(json!({"jsonrpc": "2.0", "result": ops}).to_string())
}
