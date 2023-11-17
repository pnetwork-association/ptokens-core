use common_debug_signers::{validate_debug_command_signature, DebugSignature};
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use function_name::named;
use serde_json::json;

use crate::android::{State, CORE_TYPE};

#[named]
pub fn hard_reset(debug_sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    debug!("hard resetting...");
    let h = get_debug_command_hash!(function_name!())()?;
    validate_debug_command_signature(state.db(), &CORE_TYPE, &debug_sig.to_string(), &h, cfg!(test))?;
    state.db().drop_db()?;
    Ok(state.add_response(WebSocketMessagesEncodable::Success(json!({"dbDropped": true}))))
}
