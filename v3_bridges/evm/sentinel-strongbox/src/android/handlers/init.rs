use common::BridgeSide;
use common_eth::{init_v3_host_core, init_v3_native_core, EthSubmissionMaterial};
use common_metadata::MetadataChainId;
use common_sentinel::{SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError, WebSocketMessagesInitArgs};

use crate::android::State;

pub fn init<'a>(args: WebSocketMessagesInitArgs, state: State<'a>) -> Result<State<'a>, SentinelError> {
    if args.native_block().is_none() {
        let r = WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoBlock {
            side: BridgeSide::Native,
            struct_name: "init_args".to_string(),
        });
        return Ok(state.add_response(r));
    };

    if args.host_block().is_none() {
        let r = WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoBlock {
            side: BridgeSide::Native,
            struct_name: "init_args".to_string(),
        });
        return Ok(state.add_response(r));
    };

    if let Err(e) = init_v3_host_core(
        state.db(),
        args.host_block().clone().expect("this cannot fail due to above check"),
        args.host_chain_id(),
        *args.host_confirmations(),
        *args.host_validate(),
    ) {
        let r = WebSocketMessagesEncodable::Error(e.into());
        return Ok(state.add_response(r));
    };

    if let Err(e) = init_v3_native_core(
        state.db(),
        args.native_block()
            .clone()
            .expect("this cannot fail due to above check"),
        args.native_chain_id(),
        *args.native_confirmations(),
        *args.native_validate(),
    ) {
        let r = WebSocketMessagesEncodable::Error(e.into());
        return Ok(state.add_response(r));
    };

    let r = WebSocketMessagesEncodable::Success("core initialized".into());
    Ok(state.add_response(r))
}
