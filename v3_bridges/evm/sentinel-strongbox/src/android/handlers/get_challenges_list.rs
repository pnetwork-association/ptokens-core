use common_sentinel::{ChallengesList, SentinelDbUtils, SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_challenges_list(state: State) -> Result<State, SentinelError> {
    let r = WebSocketMessagesEncodable::Success(json!(ChallengesList::get(&SentinelDbUtils::new(state.db()))));
    Ok(state.add_response(r))
}
