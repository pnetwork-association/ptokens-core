use common_eth::{Chain, ChainDbUtils};
use common_metadata::MetadataChainId;
use common_sentinel::{
    LatestBlockInfo,
    LatestBlockInfos,
    NetworkId,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use serde_json::json;

use crate::android::State;

pub fn get_latest_block_infos(network_ids: Vec<NetworkId>, state: State) -> Result<State, SentinelError> {
    let chain_db_utils = ChainDbUtils::new(state.db());
    let mut chains = vec![];

    for network_id in network_ids.iter() {
        let mcid = MetadataChainId::try_from(network_id)?;
        match Chain::get(&chain_db_utils, mcid) {
            Ok(c) => {
                chains.push(c);
            },
            Err(e) => {
                error!("{e}");
                let e = WebSocketMessagesError::NotInitialized(*network_id);
                return Ok(state.add_response(WebSocketMessagesEncodable::Error(e)));
            },
        }
    }

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
