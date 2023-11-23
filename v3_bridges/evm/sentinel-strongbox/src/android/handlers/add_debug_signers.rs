use common_debug_signers::{debug_add_multiple_debug_signers, DebugSignature};
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use ethereum_types::Address as EthAddress;
use serde_json::{json, Value as Json};

use crate::android::{State, CORE_TYPE};

pub fn add_debug_signers(
    signers: Vec<(String, EthAddress)>,
    sig: DebugSignature,
    state: State,
) -> Result<State, SentinelError> {
    // NOTE: The `debug_add_...` fxn in the debug_signers crate handleds the validation of a signature for
    // itself, so we don't have to do it in here.

    let jsons = signers
        .into_iter()
        .map(|(name, address)| json!({"name": name, "eth_address": address}))
        .collect::<Vec<Json>>();

    let signers_json = json!(jsons);

    debug_add_multiple_debug_signers(state.db(), &signers_json.to_string(), &CORE_TYPE, &sig.to_string())?;

    let msg = WebSocketMessagesEncodable::Success(json!({"debugSignersAdded": signers_json}));

    Ok(state.add_response(msg))
}
