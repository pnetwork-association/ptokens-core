use common_sentinel::{ChallengesList, SentinelDbUtils, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::H256 as EthHash;
use serde_json::json;

use crate::android::State;

pub fn get_challenge(hash: EthHash, state: State) -> Result<State, SentinelError> {
    let db_utils = SentinelDbUtils::new(state.db());
    let list = ChallengesList::get(&db_utils);
    let challenge = list.get_challenge(&db_utils, &hash)?;

    let r = WebSocketMessagesEncodable::Success(json!(challenge));
    Ok(state.add_response(r))
}
