use common_sentinel::{
    ActorInclusionProof,
    SentinelDbUtils,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use serde_json::json;

use crate::android::State;

pub fn get_inclusion_proof(state: State) -> Result<State, SentinelError> {
    let p = ActorInclusionProof::get(&SentinelDbUtils::new(state.db()));
    if p == ActorInclusionProof::default() {
        let r = WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoInclusionProof);
        Ok(state.add_response(r))
    } else {
        let r = WebSocketMessagesEncodable::Success(json!(ActorInclusionProof::get(&SentinelDbUtils::new(state.db()))));
        Ok(state.add_response(r))
    }
}
