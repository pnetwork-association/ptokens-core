use common_metadata::MetadataChainId;
use common_sentinel::{CoreState, SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_core_state(mcids: Vec<MetadataChainId>, state: State) -> Result<State, SentinelError> {
    let r = WebSocketMessagesEncodable::Success(json!(CoreState::get(state.db(), mcids)?));
    Ok(state.add_response(r))
}
