use common_sentinel::{ChallengesList, SentinelDbUtils, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::H256 as EthHash;
use serde_json::json;

use crate::android::State;

pub fn set_challenges_to_solved(ids: Vec<EthHash>, state: State) -> Result<State, SentinelError> {
    debug!("setting challenge status to solved for ids: {ids:?}");
    let s_db_utils = SentinelDbUtils::new(state.db());

    let mut list = ChallengesList::get(&s_db_utils);
    list.update_challenge_statuses_to_solved(&s_db_utils, ids.clone())?;

    let r = WebSocketMessagesEncodable::Success(json!({"challengesSetToSolved": ids}));
    Ok(state.add_response(r))
}
