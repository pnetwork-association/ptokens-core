use common_sentinel::{ActorInclusionProof, SentinelDbUtils, SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_inclusion_proof(state: State) -> Result<State, SentinelError> {
    let r = WebSocketMessagesEncodable::Success(json!(ActorInclusionProof::get(&SentinelDbUtils::new(state.db()))));
    Ok(state.add_response(r))
}
