use common_sentinel::{SentinelDbUtils, SentinelError, UserOpList, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::{strongbox::Strongbox, State};

pub fn get_attestation(state: State) -> Result<State, SentinelError> {
    let j = json!({"success": state.strongbox().check_keystore_is_initialized()? });

    let r = WebSocketMessagesEncodable::Success(j);
    Ok(state.add_response(r))
}
