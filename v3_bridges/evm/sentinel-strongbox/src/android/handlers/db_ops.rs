use common::MIN_DATA_SENSITIVITY_LEVEL;
use common_sentinel::{SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

// TODO/FIXME: Handle different data sensitivities
type Bytes = Vec<u8>;

fn to_prefixed_hex_string(bs: &[u8]) -> String {
    format!("0x{}", hex::encode(bs))
}

pub fn get(k: Bytes, state: State) -> Result<State, SentinelError> {
    let v = state.db().get(&k, MIN_DATA_SENSITIVITY_LEVEL)?;
    let msg = WebSocketMessagesEncodable::Success(json!({
        "dbOp": "get",
        "key": to_prefixed_hex_string(&k),
        "value": to_prefixed_hex_string(&v),
    }));
    Ok(state.add_response(msg))
}

pub fn put(k: Bytes, v: Bytes, state: State) -> Result<State, SentinelError> {
    let r = state.db().put(&k, &v, MIN_DATA_SENSITIVITY_LEVEL);
    let msg = WebSocketMessagesEncodable::Success(json!({
        "dbOp": "put",
        "key": to_prefixed_hex_string(&k),
        "value": to_prefixed_hex_string(&v),
        "success": r.is_ok(),
    }));
    Ok(state.add_response(msg))
}

pub fn delete(k: Bytes, state: State) -> Result<State, SentinelError> {
    let r = state.db().delete(&k);
    let msg = WebSocketMessagesEncodable::Success(json!({
        "dbOp": "delete",
        "key": to_prefixed_hex_string(&k),
        "success": r.is_ok(),
    }));
    Ok(state.add_response(msg))
}