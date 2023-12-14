use common_debug_signers::{validate_debug_command_signature, DebugSignature};
use common_sentinel::{ChallengesList, SentinelDbUtils, SentinelError, WebSocketMessagesEncodable};
use ethereum_types::H256 as EthHash;
use function_name::named;
use serde_json::json;

use crate::android::{State, CORE_TYPE};

#[named]
pub fn remove_challenge(hash: EthHash, sig: DebugSignature, state: State) -> Result<State, SentinelError> {
    let h = get_debug_command_hash!(function_name!(), &hash)()?;
    validate_debug_command_signature(state.db(), &CORE_TYPE, &sig.to_string(), &h, cfg!(test))?;
    let db_utils = SentinelDbUtils::new(state.db());
    let mut list = ChallengesList::get(&db_utils);
    list.remove_challenge(&db_utils, &hash)?;

    let r = WebSocketMessagesEncodable::Success(json!({ "removedChallenge": hash}));
    Ok(state.add_response(r))
}
