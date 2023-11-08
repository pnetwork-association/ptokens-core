use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_attestation_signature(bytes: Vec<u8>, state: State) -> Result<State, SentinelError> {
    let j = json!({
        "message": format!("0x{}", hex::encode(&bytes)),
        "attestationSignature": format!("0x{}", hex::encode(state.strongbox().get_attestation_signature(bytes)?)),
    });

    let r = WebSocketMessagesEncodable::Success(j);
    Ok(state.add_response(r))
}
