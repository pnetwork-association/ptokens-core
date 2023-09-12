use common::CoreType;
use common_eth::{init_v3_host_core, init_v3_native_core};
use common_sentinel::{SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError, WebSocketMessagesInitArgs};

use crate::android::State;

pub fn init(args: WebSocketMessagesInitArgs, state: State) -> Result<State, SentinelError> {
    if CoreType::host_core_is_initialized(state.db()) {
        return Err(WebSocketMessagesError::AlreadyInitialized(args.host_chain_id().clone()).into());
    };

    if CoreType::native_core_is_initialized(state.db()) {
        return Err(WebSocketMessagesError::AlreadyInitialized(args.native_chain_id().clone()).into());
    };

    init_v3_host_core(
        state.db(),
        args.to_host_sub_mat()?,
        args.host_chain_id(),
        *args.host_confirmations(),
        *args.host_validate(),
    )?;

    init_v3_native_core(
        state.db(),
        args.to_native_sub_mat()?,
        args.native_chain_id(),
        *args.native_confirmations(),
        *args.native_validate(),
    )?;

    let r = WebSocketMessagesEncodable::Success("core initialized".into());
    Ok(state.add_response(r))
}
