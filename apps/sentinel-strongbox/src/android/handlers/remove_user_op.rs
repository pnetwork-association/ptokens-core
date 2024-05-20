use common_debug_signers::{validate_debug_command_signature, DebugSignature};
use common_sentinel::{SentinelDbUtils, SentinelError, UserOpList, UserOpUniqueId, WebSocketMessagesEncodable};
use function_name::named;
use serde_json::json;

use crate::android::{State, CORE_TYPE};

#[named]
pub fn remove_user_op(uid: UserOpUniqueId, sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    let h = get_debug_command_hash!(function_name!(), &uid)()?;
    validate_debug_command_signature(state.db(), &CORE_TYPE, &sig.to_string(), &h, cfg!(test))?;
    let db_utils = SentinelDbUtils::new(state.db());
    let mut list = UserOpList::get(&db_utils);
    let was_removed = list.remove_entry(&db_utils, &uid)?;
    let r = WebSocketMessagesEncodable::Success(json!({ "uid": uid, "removed_from_list": was_removed }));
    Ok(state.add_response(r))
}
