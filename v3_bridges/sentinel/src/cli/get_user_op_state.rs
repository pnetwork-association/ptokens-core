use common_eth::convert_hex_to_h256;
use common_rocksdb_database::get_db_at_path;
use lib::{
    check_init,
    get_user_op_state as get_user_op_state_rpc_call,
    ConfigT,
    DbUtilsT,
    SentinelConfig,
    SentinelDbUtils,
    SentinelError,
    UserOp,
    DEFAULT_SLEEP_TIME,
};
use serde_json::json;

#[derive(Clone, Debug, Default, Args)]
pub struct GetUserOpStateCliArgs {
    /// User op identifaction hash
    uid: String,
}

pub async fn get_user_op_state(config: &SentinelConfig, args: &GetUserOpStateCliArgs) -> Result<String, SentinelError> {
    let db = get_db_at_path(&config.get_db_path())?;
    let db_utils = SentinelDbUtils::new(&db);
    check_init(&db)?;

    let uid = convert_hex_to_h256(&args.uid)?;
    match UserOp::get_from_db(&db_utils, &uid.into()) {
        Err(e) => {
            warn!("{e}");
            Err(SentinelError::Custom(format!("no user op in db with uid {uid}")))
        },
        Ok(op) => {
            let side = op.destination_side();

            let state_manager = if side.is_native() {
                config.native().state_manager()
            } else {
                config.host().state_manager()
            };

            let ws_client = if side.is_native() {
                config.native().endpoints().get_first_ws_client().await?
            } else {
                config.host().endpoints().get_first_ws_client().await?
            };

            let state = get_user_op_state_rpc_call(&op, &state_manager, &ws_client, DEFAULT_SLEEP_TIME, side).await?;

            let r = json!({"jsonrpc": "2.0", "result": { "user_op_state": state.to_string(), "uid": args.uid }})
                .to_string();

            Ok(r)
        },
    }
}
