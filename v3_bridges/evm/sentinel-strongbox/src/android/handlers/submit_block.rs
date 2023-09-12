use common::{BridgeSide, CoreType};
use common_eth::{init_v3_host_core, init_v3_native_core, EthSubmissionMaterial};
use common_metadata::MetadataChainId;
use common_sentinel::{
    process_single,
    NetworkId,
    ProtocolId,
    SentinelError,
    UserOps,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesSubmitArgs,
};
use derive_more::Constructor;
use serde::Serialize;
use serde_json::{json, Value as Json};

use crate::android::State;

#[derive(Serialize, Constructor)]
struct Response {
    side: BridgeSide,
    user_ops: UserOps,
    num: u64,
}

impl Into<Json> for Response {
    fn into(self) -> Json {
        json!(self)
    }
}

pub fn submit_block(args: WebSocketMessagesSubmitArgs, state: State) -> Result<State, SentinelError> {
    let use_db_tx = !args.dry_run();

    let user_ops = process_single(
        state.db(),
        args.sub_mat(),
        args.pnetwork_hub(),
        *args.validate(),
        use_db_tx,
        *args.dry_run(),
        *args.side(),
        &NetworkId::new(args.eth_chain_id().clone(), ProtocolId::Ethereum).to_bytes_4()?,
        *args.reprocess(),
    )?;

    let r = WebSocketMessagesEncodable::Success(
        Response::new(
            args.side().clone(),
            user_ops,
            args.sub_mat().get_block_number()?.as_u64(),
        )
        .into(),
    );
    Ok(state.add_response(r))
}
