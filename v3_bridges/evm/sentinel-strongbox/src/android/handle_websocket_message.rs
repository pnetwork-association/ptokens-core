use std::result::Result;

use common_metadata::MetadataChainId;
use common_sentinel::{check_init, SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError};

use super::State;

pub fn handle_websocket_message(state: State) -> Result<State, SentinelError> {
    info!("handling web socket message...");
    state.db().start_transaction()?;
    let msg = state.msg();

    match &msg {
        WebSocketMessagesEncodable::Initialize(_) => {
            warn!("skipping init check");
            // NOTE: We skip the init check if we actually trying to initialize a core.
            Ok(())
        },
        _ => check_init(state.db()),
    }?;

    let final_state = match msg {
        WebSocketMessagesEncodable::GetCoreState => super::handlers::get_core_state(state),
        WebSocketMessagesEncodable::Initialize(args) => super::handlers::init(*args.clone(), state),
        WebSocketMessagesEncodable::Submit(args) => super::handlers::submit_block(*args.clone(), state),
        WebSocketMessagesEncodable::GetLatestBlockNumbers => super::handlers::get_latest_block_numbers(state),
        m => Err(WebSocketMessagesError::Unhandled(m.to_string()).into()),
    }?;

    final_state.db().end_transaction()?;
    Ok(final_state)
}
