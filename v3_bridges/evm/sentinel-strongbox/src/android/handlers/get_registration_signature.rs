use common_eth::ChainDbUtils;
use common_sentinel::{get_registration_signature as get_reg_sig, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::Address as EthAddress;
use serde_json::json;

use crate::android::State;

pub fn get_registration_signature(a: EthAddress, state: State) -> Result<State, SentinelError> {
    let owner = format!("0x{}", hex::encode(a));
    debug!("handling `GetRegistationSignature` for owner address {owner} in strongbox...");
    let chain_db_utils = ChainDbUtils::new(state.db());
    let pk = chain_db_utils.get_pk()?;
    let sig = get_reg_sig(&a, &pk)?;
    let signer = format!("0x{}", hex::encode(pk.to_address()));
    let json = json!({ "signer": signer, "owner": owner, "signature": format!("0x{sig}") });
    let r = WebSocketMessagesEncodable::Success(json);
    Ok(state.add_response(r))
}
