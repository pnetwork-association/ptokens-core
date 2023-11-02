use common::MIN_DATA_SENSITIVITY_LEVEL;
use common_debug_signers::validate_debug_command_signature;
use common_sentinel::{DebugSignature, SentinelError, WebSocketMessagesEncodable};
use function_name::named;
use serde_json::json;

use crate::android::{State, CORE_TYPE};

// TODO/FIXME: Handle different data sensitivities
type Bytes = Vec<u8>;

fn to_prefixed_hex_string(bs: &[u8]) -> String {
    format!("0x{}", hex::encode(bs))
}

#[named]
pub fn get(k: Bytes, sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    let h = get_debug_command_hash!(function_name!(), &k)()?;
    validate_debug_command_signature(state.db(), &CORE_TYPE, &sig.to_string(), &h, cfg!(test))?;

    let v = state.db().get(&k, MIN_DATA_SENSITIVITY_LEVEL)?;
    let msg = WebSocketMessagesEncodable::Success(json!({
        "dbOp": "get",
        "key": to_prefixed_hex_string(&k),
        "value": to_prefixed_hex_string(&v),
    }));
    Ok(state.add_response(msg))
}

#[named]
pub fn put(k: Bytes, v: Bytes, sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    let h = get_debug_command_hash!(function_name!(), &k, &v)()?;
    validate_debug_command_signature(state.db(), &CORE_TYPE, &sig.to_string(), &h, cfg!(test))?;

    let r = state.db().put(&k, &v, MIN_DATA_SENSITIVITY_LEVEL);
    let msg = WebSocketMessagesEncodable::Success(json!({
        "dbOp": "put",
        "key": to_prefixed_hex_string(&k),
        "value": to_prefixed_hex_string(&v),
        "success": r.is_ok(),
    }));
    Ok(state.add_response(msg))
}

#[named]
pub fn delete(k: Bytes, sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    let h = get_debug_command_hash!(function_name!(), &k)()?;
    validate_debug_command_signature(state.db(), &CORE_TYPE, &sig.to_string(), &h, cfg!(test))?;

    let r = state.db().delete(&k);
    let msg = WebSocketMessagesEncodable::Success(json!({
        "dbOp": "delete",
        "key": to_prefixed_hex_string(&k),
        "success": r.is_ok(),
    }));
    Ok(state.add_response(msg))
}
