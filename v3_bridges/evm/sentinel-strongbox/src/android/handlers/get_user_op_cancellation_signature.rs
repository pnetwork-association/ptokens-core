use common_eth::ChainDbUtils;
use common_sentinel::{
    ActorInclusionProof,
    SentinelDbUtils,
    SentinelError,
    WebSocketMessagesCancelUserOpArgs,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use serde_json::json;

use crate::android::State;

pub fn get_user_op_cancellation_signature(
    args: WebSocketMessagesCancelUserOpArgs,
    state: State,
) -> Result<State, SentinelError> {
    let (_, op) = args.dissolve();
    warn!("signing cancellation sig for user op {op}");
    let c_db_utils = ChainDbUtils::new(state.db());
    let s_db_utils = SentinelDbUtils::new(state.db());
    let pk = c_db_utils.get_pk()?;
    let proof = ActorInclusionProof::get(&s_db_utils);
    if proof == ActorInclusionProof::default() {
        error!("cannot cancel user op - no actor inclusion proof in db!");
        let r = WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoInclusionProof);
        return Ok(state.add_response(r));
    };

    let sig = op.get_cancellation_signature(&pk, proof)?;
    let r = WebSocketMessagesEncodable::Success(json!(sig));
    Ok(state.add_response(r))
}
