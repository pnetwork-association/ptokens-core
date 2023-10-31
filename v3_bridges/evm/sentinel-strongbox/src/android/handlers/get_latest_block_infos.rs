use common_eth::{Chain, ChainDbUtils, ChainError};
use common_metadata::MetadataChainId;
use common_sentinel::{
    LatestBlockInfo,
    LatestBlockInfos,
    NetworkId,
    NetworkIdError,
    SentinelError,
    WebSocketMessagesEncodable,
};
use serde_json::json;

use crate::android::State;

pub fn get_latest_block_infos(network_ids: Vec<NetworkId>, state: State) -> Result<State, SentinelError> {
    let chain_db_utils = ChainDbUtils::new(state.db());

    let mcids = network_ids
        .iter()
        .map(MetadataChainId::try_from)
        .collect::<Result<Vec<MetadataChainId>, NetworkIdError>>()?;

    let chains = mcids
        .iter()
        .map(|mcid| Chain::get(&chain_db_utils, *mcid))
        .collect::<Result<Vec<Chain>, ChainError>>()?;

    let latest_block_nums = chains.iter().map(|chain| *chain.offset()).collect::<Vec<u64>>();
    let latest_block_timestamps = chains
        .iter()
        .map(|chain| chain.latest_block_timestamp().as_secs())
        .collect::<Vec<u64>>();

    let infos = LatestBlockInfos::new(
        latest_block_nums
            .iter()
            .zip(latest_block_timestamps.iter())
            .enumerate()
            .map(|(i, (n, t))| LatestBlockInfo::new(*n, *t, network_ids[i]))
            .collect::<Vec<LatestBlockInfo>>(),
    );

    let r = WebSocketMessagesEncodable::Success(json!(infos));
    Ok(state.add_response(r))
}