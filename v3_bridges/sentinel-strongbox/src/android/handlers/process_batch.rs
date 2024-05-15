use common_eth::{ChainDbUtils, ChainError};
use common_sentinel::{
    process_batch as process_batch_of_blocks,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesProcessBatchArgs,
};
use serde_json::json;

use crate::android::State;

pub fn process_batch(args: WebSocketMessagesProcessBatchArgs, state: State) -> Result<State, SentinelError> {
    let network_config = args.network_config();
    let sentinel_address = ChainDbUtils::new(state.db()).get_signing_address()?;

    let result = process_batch_of_blocks(
        state.db(),
        args.pnetwork_hub(),
        args.sub_mat_batch(),
        *args.validate(),
        network_config,
        *args.reprocess(),
        *args.dry_run(),
        *args.governance_address(),
        sentinel_address,
    );

    let response = match result {
        Ok(output) => WebSocketMessagesEncodable::Success(json!(output)),
        Err(SentinelError::ChainError(ChainError::NoParent(e))) => {
            WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoParent(e))
        },
        Err(SentinelError::ChainError(ChainError::BlockAlreadyInDb { num, mcid, hash })) => {
            WebSocketMessagesEncodable::Error(WebSocketMessagesError::BlockAlreadyInDb {
                num,
                network_id: mcid.try_into()?,
                hash,
            })
        },
        Err(e) => WebSocketMessagesEncodable::Error(e.into()),
    };

    Ok(state.add_response(response))
}
