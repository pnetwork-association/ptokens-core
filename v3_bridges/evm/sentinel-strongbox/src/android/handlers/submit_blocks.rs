use common_eth::ChainError;
use common_metadata::MetadataChainId;
use common_sentinel::{
    process_batch,
    NetworkId,
    ProtocolId,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesSubmitArgs,
};
use serde_json::json;

use crate::android::State;

pub fn submit_blocks(args: WebSocketMessagesSubmitArgs, state: State) -> Result<State, SentinelError> {
    let ecid = args.eth_chain_id().clone();
    let mcid = MetadataChainId::from(&ecid);

    let result = process_batch(
        state.db(),
        args.pnetwork_hub(),
        args.sub_mat_batch(),
        *args.validate(),
        *args.side(),
        &NetworkId::new(ecid, ProtocolId::Ethereum).to_bytes_4()?,
        *args.reprocess(),
        *args.dry_run(),
        mcid,
    );

    let response = match result {
        Ok(output) => WebSocketMessagesEncodable::Success(json!(output)),
        Err(SentinelError::ChainError(ChainError::NoParent(e))) => {
            WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoParent(e))
        },
        Err(SentinelError::ChainError(ChainError::BlockAlreadyInDb { num, mcid, hash })) => {
            WebSocketMessagesEncodable::Error(WebSocketMessagesError::BlockAlreadyInDb { num, mcid, hash })
        },
        Err(e) => WebSocketMessagesEncodable::Error(e.into()),
    };

    Ok(state.add_response(response))
}
