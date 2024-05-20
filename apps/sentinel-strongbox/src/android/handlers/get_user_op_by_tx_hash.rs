use common_sentinel::{SentinelDbUtils, SentinelError, UserOpList, WebSocketMessagesEncodable};
use ethereum_types::H256 as EthHash;
use serde_json::json;

use crate::android::State;

pub fn get_user_op_by_tx_hash(tx_hash: EthHash, state: State) -> Result<State, SentinelError> {
    let ops = UserOpList::get_user_op_by_tx_hash(&tx_hash, &SentinelDbUtils::new(state.db()))?;
    let r = WebSocketMessagesEncodable::Success(json!(ops));
    Ok(state.add_response(r))
}
