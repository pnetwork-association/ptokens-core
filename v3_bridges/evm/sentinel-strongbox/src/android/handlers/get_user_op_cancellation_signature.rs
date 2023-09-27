use common_eth::{Chain, ChainDbUtils};
use common_sentinel::{
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
    let (mcids, op) = args.dissolve();
    warn!("signing cancellation sig for user op {op}");
    // FIXME currently we're still stuck using host and native, so we assume native chain ID & host chain ID are passed
    // in in that order.
    let l = mcids.len();
    let n = 2;
    if l != n {
        let r = WebSocketMessagesEncodable::Error(WebSocketMessagesError::InsufficientMcids { got: l, expected: n });
        return Ok(state.add_response(r));
    };
    let side = op.destination_side();
    let mcid = if side.is_native() { mcids[0] } else { mcids[1] };
    let c_db_utils = ChainDbUtils::new(state.db());
    let chain = Chain::get(&c_db_utils, mcid)?;
    let pk = chain.get_pk(&c_db_utils)?;
    let sig = op.get_cancellation_signature(&pk)?;
    let r = WebSocketMessagesEncodable::Success(json!(sig));
    Ok(state.add_response(r))
}
