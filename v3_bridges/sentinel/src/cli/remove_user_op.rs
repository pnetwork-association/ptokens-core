use clap::Args;
use common::DatabaseInterface;
use common_eth::convert_hex_to_h256;
use lib::{SentinelConfig, SentinelDbUtils, SentinelError, UserOpList};
use serde_json::json;

#[derive(Debug, Args)]
pub struct RemoveUserOpCliArgs {
    /// Unique ID of user operation to remove
    #[arg(long, short)]
    uid: String,
}

pub async fn remove_user_op(config: &SentinelConfig, cli_args: &RemoveUserOpCliArgs) -> Result<String, SentinelError> {
    info!("maybe removing user op...");
    let uid = convert_hex_to_h256(&cli_args.uid)?;

    if !config.core().db_exists() {
        return Err(SentinelError::Custom(format!(
            "cannot find db @ path: '{}'",
            config.core().db_path
        )));
    };
    let db = common_rocksdb_database::get_db_at_path(&config.get_db_path())?;
    let db_utils = SentinelDbUtils::new(&db);

    db.start_transaction()?;
    let mut list = UserOpList::get(&db_utils);
    let removed_from_list = list.remove_entry(&db_utils, &uid)?;
    db.end_transaction()?;

    let r = json!({
        "jsonrpc": "2.0",
        "result": { "uid": uid, "removed_from_list": removed_from_list },
    })
    .to_string();

    Ok(r)
}
