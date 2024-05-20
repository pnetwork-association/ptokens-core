use std::{process::Command, str::from_utf8};

use common::strip_hex_prefix;
use common_eth::{EthPrivateKey, EthSigningCapabilities};
use common_sentinel::SentinelError;
use serde_json::{json, Value as Json};

use crate::rpc_server::{RpcCalls, RpcParams};

fn strip_new_lines_from_str(string: String) -> String {
    string.replace('\n', "")
}

fn decrypt_pk(path: &str) -> Result<String, SentinelError> {
    debug!("decrypting private key...");
    let output = Command::new("gpg").arg("-d").arg(path).output()?;
    if !output.stdout.is_empty() {
        debug!("keyfile decrypted");
        let s = strip_new_lines_from_str(from_utf8(&output.stdout)?.into());
        Ok(s)
    } else {
        let e = from_utf8(&output.stderr)?.to_string();
        let m = format!("error decrypting keyfile: {e}");
        error!("{m}");
        let j = json!({"message": m});
        Err(SentinelError::Json(j))
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, SentinelError> {
    hex::decode(s).map_err(|e| SentinelError::Custom(format!("invalid hex error: {}", e)))
}

impl RpcCalls {
    /// Allows signing of a passed in hex message using the gpg-encrypted private key at the pass
    /// in path. NOTE: This function makes a linux command call, thus only works on linux.
    pub(crate) async fn handle_sign_message(params: RpcParams) -> Result<Json, SentinelError> {
        debug!("handling sign message...");
        let checked_params = Self::check_params(params, 2)?;
        let pk = EthPrivateKey::from_slice(&decode_hex(&decrypt_pk(&checked_params[0])?)?)?;
        let signer = format!("0x{}", hex::encode(pk.to_address().as_bytes()));
        let msg = decode_hex(&strip_hex_prefix(&checked_params[1]))?;
        let signature = pk.hash_and_sign_msg_with_eth_prefix(&msg)?;
        Ok(json!({
            "signer": signer,
            "msg": format!("0x{}", hex::encode(msg)),
            "signature": signature.to_string(),
        }))
    }
}
