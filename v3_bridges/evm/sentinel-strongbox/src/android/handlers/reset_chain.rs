use common_sentinel::{
    reset_chain as reset_chain_inner,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesResetChainArgs,
};
use serde_json::json;

use crate::android::State;

pub fn reset_chain(args: WebSocketMessagesResetChainArgs, state: State) -> Result<State, SentinelError> {
    let (confs, _, _, _, maybe_side, maybe_sub_mat) = args.dissolve();

    let side = match maybe_side {
        Some(s) => Ok(s),
        None => Err(WebSocketMessagesError::NoneError {
            arg_name: "side".into(),
            location: "WebSocketMessagesResetChainArgs".into(),
        }),
    }?;

    let sub_mat = match maybe_sub_mat {
        Some(s) => Ok(s),
        None => Err(WebSocketMessagesError::NoneError {
            arg_name: "sub_mat".into(),
            location: "WebSocketMessagesResetChainArgs".into(),
        }),
    }?;

    let output = reset_chain_inner(state.db(), confs, side, sub_mat)?;
    Ok(state.add_response(WebSocketMessagesEncodable::Success(json!(output))))
}
