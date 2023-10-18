use common_eth::{Chain, ChainDbUtils};
use common_sentinel::{
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesResetChainArgs,
};
use serde_json::json;

use crate::android::State;

pub fn reset_chain(args: WebSocketMessagesResetChainArgs, state: State) -> Result<State, SentinelError> {
    let (confs, validate, mcid, _, _, maybe_hub, _, maybe_sub_mat) = args.dissolve();
    debug!("resetting {mcid} chain...");

    let sub_mat = match maybe_sub_mat {
        Some(s) => Ok(s),
        None => Err(WebSocketMessagesError::NoneError {
            arg_name: "sub_mat".into(),
            location: "WebSocketMessagesResetChainArgs".into(),
        }),
    }?;

    let n = Chain::block_num(&sub_mat)?;

    Chain::reset(
        &ChainDbUtils::new(state.db()),
        sub_mat,
        mcid,
        validate,
        confs,
        maybe_hub,
    )?;

    Ok(state.add_response(WebSocketMessagesEncodable::Success(
        json!({"mcid": mcid, "chainResetTo": n}),
    )))
}
