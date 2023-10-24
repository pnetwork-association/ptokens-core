use common_sentinel::{CoreState, NetworkId, SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_core_state(network_ids: Vec<NetworkId>, state: State) -> Result<State, SentinelError> {
    debug!("handling `getCoreState` in strongbox...");
    let r = WebSocketMessagesEncodable::Success(json!(CoreState::get(state.db(), network_ids)?));
    Ok(state.add_response(r))
}
