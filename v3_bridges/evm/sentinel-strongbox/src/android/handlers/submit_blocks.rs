use common::BridgeSide;
use common_sentinel::{
    process_batch,
    NetworkId,
    ProcessorOutput,
    ProtocolId,
    SentinelError,
    UserOps,
    WebSocketMessagesEncodable,
    WebSocketMessagesSubmitArgs,
};
use derive_more::Constructor;
use serde::Serialize;
use serde_json::{json, Value as Json};

use crate::android::State;

pub fn submit_blocks(args: WebSocketMessagesSubmitArgs, state: State) -> Result<State, SentinelError> {
    let output = process_batch(
        state.db(),
        args.pnetwork_hub(),
        args.sub_mat_batch(),
        *args.validate(),
        *args.side(),
        &NetworkId::new(args.eth_chain_id().clone(), ProtocolId::Ethereum).to_bytes_4()?,
        *args.reprocess(),
        *args.dry_run(),
    )?;

    Ok(state.add_response(WebSocketMessagesEncodable::Success(json!(output))))
}
