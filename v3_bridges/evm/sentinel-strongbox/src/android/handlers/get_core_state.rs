use common_sentinel::{
    CoreState,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesInitArgs,
};
use serde_json::json;

use crate::android::State;

pub fn get_core_state(state: State) -> Result<State, SentinelError> {
    let r = WebSocketMessagesEncodable::Success(json!(CoreState::get(state.db())?));
    Ok(state.add_response(r))
}
