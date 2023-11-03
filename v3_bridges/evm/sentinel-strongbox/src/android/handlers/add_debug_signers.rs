use common_debug_signers::{
    debug_add_multiple_debug_signers,
    validate_debug_command_signature,
    DebugSignatories,
    DebugSignature,
};
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use ethereum_types::Address as EthAddress;
use function_name::named;
use serde_json::{json, Value as Json};

use crate::android::{State, CORE_TYPE};

#[named]
pub fn add_debug_signers(
    signers: Vec<(String, EthAddress)>,
    sig: DebugSignature,
    state: State,
) -> Result<State, SentinelError> {
    if !DebugSignatories::get_from_db(state.db())?.is_empty() {
        warn!("skipping debug hash validating because there are no debug signers yet!");
        let h = get_debug_command_hash!(function_name!(), &signers)()?;
        validate_debug_command_signature(state.db(), &CORE_TYPE, &sig.to_string(), &h, cfg!(test))?;
    };

    let jsons = signers
        .into_iter()
        .map(|(name, address)| json!({"name": name, "eth_address": address}))
        .collect::<Vec<Json>>();
    let signers_json = json!(jsons);

    debug_add_multiple_debug_signers(state.db(), &signers_json.to_string(), &CORE_TYPE, &sig.to_string())?;

    let msg = WebSocketMessagesEncodable::Success(json!({"debugSignersAdded": signers_json}));

    Ok(state.add_response(msg))
}
