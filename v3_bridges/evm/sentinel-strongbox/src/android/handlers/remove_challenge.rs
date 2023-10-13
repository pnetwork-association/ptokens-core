use common_sentinel::{ChallengesList, SentinelDbUtils, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::H256 as EthHash;
use serde_json::json;

use crate::android::State;

pub fn remove_challenge(hash: EthHash, state: State) -> Result<State, SentinelError> {
    let db_utils = SentinelDbUtils::new(state.db());
    let mut list = ChallengesList::get(&db_utils);
    list.remove_challenge(&db_utils, &hash)?;

    let r = WebSocketMessagesEncodable::Success(json!({ "removedChallenge": hash}));
    Ok(state.add_response(r))
}
