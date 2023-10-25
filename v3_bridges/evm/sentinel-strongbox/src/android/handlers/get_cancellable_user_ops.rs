use common_eth::{Chain, ChainDbUtils, ChainError};
use common_metadata::MetadataChainId;
use common_sentinel::{
    LatestBlockInfo,
    LatestBlockInfos,
    NetworkIdError,
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
    let (max_delta, network_ids) = args.dissolve();
    let mcids = network_ids
        .iter()
        .map(MetadataChainId::try_from)
        .collect::<Result<Vec<MetadataChainId>, NetworkIdError>>()?;
    let c_db_utils = ChainDbUtils::new(state.db());
    let s_db_utils = SentinelDbUtils::new(state.db());
    let chains = mcids
        .iter()
        .map(|mcid| Chain::get(&c_db_utils, *mcid))
        .collect::<Result<Vec<Chain>, ChainError>>()?;

    let latest_block_nums = chains.iter().map(|chain| *chain.offset()).collect::<Vec<u64>>();

    let latest_block_timestamps = chains
        .iter()
        .map(|c| c.latest_block_timestamp().as_secs())
        .collect::<Vec<u64>>();

    let infos = LatestBlockInfos::new(
        latest_block_nums
            .iter()
            .zip(latest_block_timestamps.iter())
            .enumerate()
            .map(|(i, (n, t))| LatestBlockInfo::new(*n, *t, network_ids[i]))
            .collect::<Vec<LatestBlockInfo>>(),
    );

    // NOTE: We need at least two chains in order to make useful decisions for user ops
    // cancellation eligibility.
    let min_num_chains = 2;

    let response = if latest_block_timestamps.len() != min_num_chains {
        WebSocketMessagesEncodable::Error(WebSocketMessagesError::InsufficientMcids {
            got: mcids.len(),
            expected: min_num_chains,
        })
    } else {
        let list = UserOpList::get(&s_db_utils);
        debug!("user op list: {list}");

        let cancellable_ops = list.get_cancellable_ops(max_delta, &s_db_utils, infos)?;
        debug!("cancellable ops: {cancellable_ops}");

        WebSocketMessagesEncodable::Success(json!(cancellable_ops))
    };

    Ok(state.add_response(response))
}
