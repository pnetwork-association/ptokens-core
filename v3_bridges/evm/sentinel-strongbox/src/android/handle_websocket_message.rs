use std::result::Result;
use common_sentinel::SentinelError;
use super::State;

pub fn handle_websocket_message(state: State) -> Result<State, SentinelError> {
    info!("handling web socket message...");
    Ok(state)
}
