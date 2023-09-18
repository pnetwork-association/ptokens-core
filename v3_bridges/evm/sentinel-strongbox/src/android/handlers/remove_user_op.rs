use common_sentinel::{SentinelDbUtils, SentinelError, UserOpList, UserOpUniqueId, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn remove_user_op(uid: UserOpUniqueId, state: State) -> Result<State, SentinelError> {
    let db_utils = SentinelDbUtils::new(state.db());
    let mut list = UserOpList::get(&db_utils);
    let was_removed = list.remove_entry(&db_utils, &uid)?;
    let r = WebSocketMessagesEncodable::Success(json!({ "uid": uid, "removed_from_list": was_removed }));
    Ok(state.add_response(r))
}
