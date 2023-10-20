use common_eth::ChainDbUtils;
use common_sentinel::{
    ActorInclusionProof,
    ChallengesList,
    SentinelDbUtils,
    SentinelError,
    WebSocketMessagesEncodable,
};
use serde_json::json;

use crate::android::State;

pub fn get_unsolved_challenges(state: State) -> Result<State, SentinelError> {
    let c_db_utils = ChainDbUtils::new(state.db());
    let s_db_utils = SentinelDbUtils::new(state.db());
    let signing_key = c_db_utils.get_pk()?;
    let inclusion_proof = ActorInclusionProof::get(&s_db_utils);

    let unsolved_challenges =
        ChallengesList::get_unsolved_challenges_with_signature_info(&s_db_utils, &signing_key, &inclusion_proof)?;

    let r = WebSocketMessagesEncodable::Success(json!(unsolved_challenges));
    Ok(state.add_response(r))
}
