use common_sentinel::{SentinelDbUtils, SentinelError, UserOpList, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_user_ops(state: State) -> Result<State, SentinelError> {
    let r = WebSocketMessagesEncodable::Success(json!(UserOpList::user_ops(&SentinelDbUtils::new(state.db()))?));
    Ok(state.add_response(r))
}
