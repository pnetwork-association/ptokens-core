use common_eth::{Chain, ChainDbUtils};
use common_metadata::MetadataChainId;
use common_network_ids::NetworkId;
use common_sentinel::{SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError};
use serde_json::json;

use crate::android::State;

pub fn check_init(network_id: NetworkId, state: State) -> Result<State, SentinelError> {
    let mcid = MetadataChainId::try_from(network_id)?;

    let is_initialized = Chain::is_initialized(&ChainDbUtils::new(state.db()), mcid);

    let r = if is_initialized {
        WebSocketMessagesEncodable::Success(json!({"network_id": network_id, "coreInitialized": is_initialized}))
    } else {
        WebSocketMessagesEncodable::Error(WebSocketMessagesError::NotInitialized(network_id))
    };

    Ok(state.add_response(r))
}
