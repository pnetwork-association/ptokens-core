use common_eth::EthPrivateKey;
use lib::SentinelError;
use serde_json::json;

pub fn generate_private_key() -> Result<String, SentinelError> {
    let pk = EthPrivateKey::generate_random()?;
    let r = json!({
        "jsonrpc": "2.0",
        "result": { "private_key": pk.to_hex(), "address": pk.to_address(), }
    })
    .to_string();

    Ok(r)
}
