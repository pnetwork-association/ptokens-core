use common_eth::{Chain, ChainDbUtils, ChainError};
use common_metadata::MetadataChainId;
use common_sentinel::{
    LatestBlockInfo,
    LatestBlockInfos,
    NetworkIdError,
    SentinelConfig,
    SentinelDbUtils,
    SentinelError,
    UserOpList,
    WebSocketMessagesEncodable,
};
use serde_json::json;

use crate::android::State;

pub fn get_cancellable_user_ops(config: SentinelConfig, state: State) -> Result<State, SentinelError> {
    debug!("handling cancellable user ops in core...");

    let network_ids = config.network_ids();

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

    let list = UserOpList::get(&s_db_utils);
    debug!("user op list: {list}");

    let cancellable_ops = list.get_cancellable_ops(&config, &s_db_utils, infos)?;
    debug!("cancellable ops: {cancellable_ops}");

    let response = WebSocketMessagesEncodable::Success(json!(cancellable_ops));

    Ok(state.add_response(response))
}
