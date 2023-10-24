use common_eth::{Chain, ChainDbUtils};
use common_metadata::MetadataChainId;
use common_sentinel::{
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesResetChainArgs,
};
use serde_json::json;

use crate::android::State;

pub fn reset_chain(args: WebSocketMessagesResetChainArgs, state: State) -> Result<State, SentinelError> {
    let (confs, validate, network_id, _, _, maybe_hub, maybe_sub_mat) = args.dissolve();
    let mcid = MetadataChainId::try_from(network_id)?;
    debug!("resetting {network_id} chain...");

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
