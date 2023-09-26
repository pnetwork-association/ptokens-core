use common_eth::{Chain, ChainDbUtils, ChainError};
use common_sentinel::{
    SentinelDbUtils,
    SentinelError,
    UserOpList,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesGetCancellableUserOpArgs,
};
use serde_json::json;

use crate::android::State;

pub fn get_cancellable_user_ops(
    args: WebSocketMessagesGetCancellableUserOpArgs,
    state: State,
) -> Result<State, SentinelError> {
    debug!("handling cancellable user ops in core...");
    let (max_delta, mcids) = args.dissolve();
    let c_db_utils = ChainDbUtils::new(state.db());
    let s_db_utils = SentinelDbUtils::new(state.db());
    let chains = mcids
        .iter()
        .map(|mcid| Chain::get(&c_db_utils, *mcid))
        .collect::<Result<Vec<Chain>, ChainError>>()?;
    let latest_block_timestamps = chains
        .iter()
        .map(|c| c.latest_block_timestamp().as_secs())
        .collect::<Vec<u64>>();

    // NOTE: We need at least two chains in order to make useful decisions for user ops
    // cancellation eligibility.
    let min_num_chains = 2;

    let response = if latest_block_timestamps.len() != min_num_chains {
        WebSocketMessagesEncodable::Error(WebSocketMessagesError::InsufficientMcids {
            got: mcids.len(),
            expected: min_num_chains,
        })
    } else {
        // FIXME We currently assume the first one passed in to be native and the second to be host.
        // Future changes will get rid of this foot gun.
        let n_latest_block_timestamp = latest_block_timestamps[0];
        let h_latest_block_timestamp = latest_block_timestamps[1];

        let list = UserOpList::get(&s_db_utils);
        debug!("user op list: {list}");

        let cancellable_ops = list.get_cancellable_ops(
            max_delta,
            &s_db_utils,
            n_latest_block_timestamp,
            h_latest_block_timestamp,
        )?;
        debug!("cancellable ops: {cancellable_ops}");

        WebSocketMessagesEncodable::Success(json!(cancellable_ops))
    };

    Ok(state.add_response(response))
}
