use common_debug_signers::{validate_debug_command_signature, DebugSignature};
use common_eth::ChainDbUtils;
use common_sentinel::{get_registration_signature as get_reg_sig, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::Address as EthAddress;
use function_name::named;
use serde_json::json;

use crate::android::{State, CORE_TYPE};

#[named]
pub fn get_registration_signature(
    a: EthAddress,
    n: u64,
    sig: DebugSignature,
    state: State,
) -> Result<State, SentinelError> {
    let h = get_debug_command_hash!(function_name!(), &a, &n)()?;
    validate_debug_command_signature(state.db(), &CORE_TYPE, &sig.to_string(), &h, cfg!(test))?;

    let owner = format!("0x{}", hex::encode(a));
    debug!("handling `GetRegistationSignature` for owner address {owner} in strongbox...");
    let chain_db_utils = ChainDbUtils::new(state.db());
    let pk = chain_db_utils.get_pk()?;
    let sig = get_reg_sig(&a, n, &pk)?;
    let signer = format!("0x{}", hex::encode(pk.to_address()));
    let json = json!({ "signer": signer, "owner": owner, "nonce": n, "signature": format!("0x{sig}") });
    let r = WebSocketMessagesEncodable::Success(json);
    Ok(state.add_response(r))
}
