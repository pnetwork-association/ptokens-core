use common_debug_signers::{debug_add_multiple_debug_signers_with_options, DebugSignature};
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use ethereum_types::Address as EthAddress;
use serde_json::{json, Value as Json};

use crate::android::{State, CORE_TYPE};

pub fn add_debug_signers(
    signers: Vec<(String, EthAddress)>,
    sig: DebugSignature,
    state: State,
) -> Result<State, SentinelError> {
    let jsons = signers
        .into_iter()
        .map(|(name, address)| json!({"name": name, "eth_address": address}))
        .collect::<Vec<Json>>();

    let signers_json = json!(jsons);

    // NOTE: The `debug_add_...` fxn in the debug_signers crate handleds the validation of a signature for
    // itself, so we don't have to do it in here. It also gives us the option to use db txs which
    // we don't want since that'll cause duplicate tx errors in java, and we don't want to use the
    // safe addresses for initial validation since they're not relevant in v3 bridges.

    let use_db_tx = false;
    let use_safe_addresses = false;

    debug_add_multiple_debug_signers_with_options(
        state.db(),
        &signers_json.to_string(),
        &CORE_TYPE,
        &sig.to_string(),
        use_safe_addresses,
        use_db_tx,
    )?;

    let msg = WebSocketMessagesEncodable::Success(json!({"debugSignersAdded": signers_json}));

    Ok(state.add_response(msg))
}
