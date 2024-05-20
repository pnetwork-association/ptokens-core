use common_sentinel::{SentinelDbUtils, SentinelError, UserOpList, UserOpUniqueId, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_user_op(uid: UserOpUniqueId, state: State) -> Result<State, SentinelError> {
    let db_utils = SentinelDbUtils::new(state.db());
    let op = UserOpList::user_op(&uid, &db_utils)?;
    let r = WebSocketMessagesEncodable::Success(json!(op));
    Ok(state.add_response(r))
}
