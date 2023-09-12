use common_eth::{EthDbUtilsExt, HostDbUtils, NativeDbUtils};
use common_sentinel::{LatestBlockNumber, LatestBlockNumbers, SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_latest_block_numbers(state: State) -> Result<State, SentinelError> {
    let h_db_utils = HostDbUtils::new(state.db());
    let n_db_utils = NativeDbUtils::new(state.db());

    let r = LatestBlockNumbers::new(vec![
        LatestBlockNumber::new((
            h_db_utils.get_eth_chain_id_from_db()?,
            h_db_utils.get_latest_eth_block_number()? as u64,
        )),
        LatestBlockNumber::new((
            n_db_utils.get_eth_chain_id_from_db()?,
            n_db_utils.get_latest_eth_block_number()? as u64,
        )),
    ]);

    let r = WebSocketMessagesEncodable::Success(json!(r));
    Ok(state.add_response(r))
}
