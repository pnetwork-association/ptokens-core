use std::result::Result;

use common_metadata::MetadataChainId;
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};

use super::State;

// TODO handle msg, put result in state. If no string to return, just return success WSM
pub fn handle_websocket_message(state: State) -> Result<State, SentinelError> {
    info!("handling web socket message...");
    state.db().start_transaction()?;
    let msg = state.msg();
    let final_state = match msg {
        WebSocketMessagesEncodable::Initialize(args) => super::handlers::init(*args.clone(), state),
        _ => todo!("return an error saying that we can't handle wsm: {msg}"),
    }?;
    final_state.db().end_transaction()?;
    Ok(final_state)
}
