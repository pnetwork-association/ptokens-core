use common_eth::{Chain, ChainDbUtils, ChainError};
use common_metadata::MetadataChainId;
use common_sentinel::{LatestBlockNumber, LatestBlockNumbers, SentinelError, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_latest_block_numbers(mcids: Vec<MetadataChainId>, state: State) -> Result<State, SentinelError> {
    let chain_db_utils = ChainDbUtils::new(state.db());

    let chains = mcids
        .iter()
        .map(|mcid| Chain::get(&chain_db_utils, *mcid))
        .collect::<Result<Vec<Chain>, ChainError>>()?;

    let latest_block_nums = chains.iter().map(|chain| *chain.offset()).collect::<Vec<u64>>();

    let nums = LatestBlockNumbers::new(
        latest_block_nums
            .iter()
            .enumerate()
            .map(|(i, n)| LatestBlockNumber::new((mcids[i], *n)))
            .collect::<Vec<LatestBlockNumber>>(),
    );

    let r = WebSocketMessagesEncodable::Success(json!(nums));
    Ok(state.add_response(r))
}
