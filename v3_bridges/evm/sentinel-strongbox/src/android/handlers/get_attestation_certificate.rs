use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_attestation_certificate(state: State) -> Result<State, SentinelError> {
    let j = json!({"attestionCertificate": state.strongbox().get_attestation_certificate()? });

    let r = WebSocketMessagesEncodable::Success(j);
    Ok(state.add_response(r))
}
