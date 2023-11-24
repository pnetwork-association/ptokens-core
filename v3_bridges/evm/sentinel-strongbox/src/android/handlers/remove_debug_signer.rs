use common_debug_signers::{debug_remove_debug_signer, DebugSignature};
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::{State, CORE_TYPE};

pub fn remove_debug_signer(signer: String, sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    // NOTE: The `debug_remove_...` fxn in the debug_signers crate handleds the validation of a signature for
    // itself, so we don't have to do it in here.

    debug_remove_debug_signer(state.db(), &signer, &CORE_TYPE, &sig.to_string())?;

    let msg = WebSocketMessagesEncodable::Success(json!({"debugSignerRemoved": signer}));

    Ok(state.add_response(msg))
}
