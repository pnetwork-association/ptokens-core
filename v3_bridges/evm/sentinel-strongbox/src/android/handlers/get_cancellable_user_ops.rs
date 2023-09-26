use common_eth::{EthDbUtilsExt, HostDbUtils, NativeDbUtils};
use common_sentinel::{SentinelDbUtils, SentinelError, UserOpList, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_cancellable_user_ops(max_delta: u64, state: State) -> Result<State, SentinelError> {
    let h_db_utils = HostDbUtils::new(state.db());
    let n_db_utils = NativeDbUtils::new(state.db());
    let s_db_utils = SentinelDbUtils::new(state.db());

    let n_latest_block_timestamp = n_db_utils.get_latest_eth_block_timestamp()?;
    let h_latest_block_timestamp = h_db_utils.get_latest_eth_block_timestamp()?;

    let list = UserOpList::get(&s_db_utils);
    debug!("user op list: {list}");

    let cancellable_ops = list.get_cancellable_ops(
        max_delta,
        &s_db_utils,
        n_latest_block_timestamp,
        h_latest_block_timestamp,
    )?;
    debug!("cancellable ops: {cancellable_ops}");

    let r = WebSocketMessagesEncodable::Success(json!(cancellable_ops));

    Ok(state.add_response(r))
}
