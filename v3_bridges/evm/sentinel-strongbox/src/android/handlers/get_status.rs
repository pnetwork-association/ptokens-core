use common_eth::{Chain, ChainDbUtils};
use common_metadata::MetadataChainId;
use common_sentinel::{SentinelError, SentinelStatus, WebSocketMessagesEncodable};
use serde_json::json;

use crate::android::State;

pub fn get_status(mcids: Vec<MetadataChainId>, state: State) -> Result<State, SentinelError> {
    debug!("handling `getStatus` message in strongbox...");
    let db_utils = ChainDbUtils::new(state.db());
    let chains = mcids
        .iter()
        .map(|mcid| Chain::get(&db_utils, *mcid).map_err(|e| e.into()))
        .collect::<Result<Vec<Chain>, SentinelError>>()?;

    let key = db_utils.get_pk()?;

    let status = SentinelStatus::new(&key, chains)?;

    let r = WebSocketMessagesEncodable::Success(json!(status));
    Ok(state.add_response(r))
}
