use common_eth::{Chain, ChainDbUtils};
use common_sentinel::{SentinelError, WebSocketMessagesEncodable, WebSocketMessagesInitArgs};
use serde_json::json;

use crate::android::State;

pub fn init(args: WebSocketMessagesInitArgs, state: State) -> Result<State, SentinelError> {
    let mcid = *args.mcid();
    Chain::init(
        &ChainDbUtils::new(state.db()),
        *args.hub(),
        *args.tail_length(),
        *args.confirmations(),
        args.sub_mat()?,
        mcid,
        *args.validate(),
    )?;
    let r = WebSocketMessagesEncodable::Success(json!({"mcid": mcid, "coreInitialized": true}));
    Ok(state.add_response(r))
}
