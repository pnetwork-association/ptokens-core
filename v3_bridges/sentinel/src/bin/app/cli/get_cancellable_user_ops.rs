use common_eth::{EthDbUtilsExt, HostDbUtils, NativeDbUtils};
use common_rocksdb_database::get_db_at_path;
use lib::{check_init, SentinelConfig, SentinelDbUtils, SentinelError, UserOpList};
use serde_json::json;

pub async fn get_cancellable_user_ops(config: &SentinelConfig) -> Result<String, SentinelError> {
    let db = get_db_at_path(&config.get_db_path())?;
    check_init(&db)?;

    let h_db_utils = HostDbUtils::new(&db);
    let n_db_utils = NativeDbUtils::new(&db);
    let s_db_utils = SentinelDbUtils::new(&db);

    let max_delta = config.core().max_cancellable_time_delta();
    let n_latest_block_timestamp = n_db_utils.get_latest_eth_block_timestamp()?;
    let h_latest_block_timestamp = h_db_utils.get_latest_eth_block_timestamp()?;

    let list = UserOpList::get(&s_db_utils);
    let cancellable_ops = list.get_cancellable_ops(
        max_delta,
        &s_db_utils,
        n_latest_block_timestamp,
        h_latest_block_timestamp,
    )?;
    Ok(json!({ "jsonrpc": "2.0", "result": { "cancellable_ops": cancellable_ops } }).to_string())
}
