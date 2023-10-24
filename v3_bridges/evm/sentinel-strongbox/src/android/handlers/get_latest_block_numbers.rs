use common_eth::{Chain, ChainDbUtils, ChainError};
use common_metadata::MetadataChainId;
use common_sentinel::{
    LatestBlockNumber,
    LatestBlockNumbers,
    NetworkId,
    NetworkIdError,
    SentinelError,
    WebSocketMessagesEncodable,
};
use serde_json::json;

use crate::android::State;

pub fn get_latest_block_numbers(network_ids: Vec<NetworkId>, state: State) -> Result<State, SentinelError> {
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

    let nums = LatestBlockNumbers::new(
        latest_block_nums
            .iter()
            .enumerate()
            .map(|(i, n)| LatestBlockNumber::new((network_ids[i], *n)))
            .collect::<Vec<LatestBlockNumber>>(),
    );

    let r = WebSocketMessagesEncodable::Success(json!(nums));
    Ok(state.add_response(r))
}
