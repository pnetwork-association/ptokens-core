use common_debug_signers::{validate_debug_command_signature, DebugSignature};
use common_sentinel::{SentinelDbUtils, SentinelError, UserOpList, WebSocketMessagesEncodable};
use function_name::named;
use serde_json::json;

use crate::android::{State, CORE_TYPE};

#[named]
pub fn purge_user_ops(epoch: usize, sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    let h = get_debug_command_hash!(function_name!(), &epoch)()?;
    validate_debug_command_signature(state.db(), &CORE_TYPE, &sig.to_string(), &h, cfg!(test))?;
    let db_utils = SentinelDbUtils::new(state.db());
    let mut list = UserOpList::get(&db_utils);
    let n_user_ops = list.len();
    list.purge(&db_utils)?;
    let r = WebSocketMessagesEncodable::Success(json!({ "result": format!("purged {n_user_ops} user ops") }));
    Ok(state.add_response(r))
}
