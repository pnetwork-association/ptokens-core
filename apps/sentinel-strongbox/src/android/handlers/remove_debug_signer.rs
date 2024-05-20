use common_debug_signers::{debug_remove_debug_signer_with_options, DebugSignature};
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::{State, CORE_TYPE};

pub fn remove_debug_signer(signer: String, sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    // NOTE: The `debug_remove_...` fxn in the debug_signers crate handleds the validation of a signature for
    // itself, so we don't have to do it in here. It also gives us the option to use db txs which
    // we don't want since that'll cause duplicate tx errors in java.
    let use_db_tx = false;

    debug_remove_debug_signer_with_options(state.db(), &signer, &CORE_TYPE, &sig.to_string(), use_db_tx)?;

    let msg = WebSocketMessagesEncodable::Success(json!({"debugSignerRemoved": signer}));

    Ok(state.add_response(msg))
}
