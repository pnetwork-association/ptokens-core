use common_eth::{EthDbUtilsExt, NativeDbUtils};
use common_sentinel::{SentinelError, UserOp, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_user_op_cancellation_signature(op: UserOp, state: State) -> Result<State, SentinelError> {
    warn!("signing cancellation sig for user op {op}");
    // NOTE: v3 cores don't need two keys, just the one. So we'll use the native one for now.
    let n_db_utils = NativeDbUtils::new(state.db());
    let pk = n_db_utils.get_eth_private_key_from_db()?;
    let sig = op.get_cancellation_signature(&pk)?;
    let r = WebSocketMessagesEncodable::Success(json!(sig));
    Ok(state.add_response(r))
}
