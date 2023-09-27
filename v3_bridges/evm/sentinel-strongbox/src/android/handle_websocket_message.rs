use std::result::Result;

use common_sentinel::{
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesEncodableDbOps,
    WebSocketMessagesError,
};

use super::State;

pub fn handle_websocket_message(state: State) -> Result<State, SentinelError> {
    info!("handling web socket message...");

    match state.db().start_transaction() {
        Err(e) => {
            error!("error starting db tx: {e}");
            state.env().exception_describe().expect("this not to fail");
            state.env().exception_clear().expect("this not to fail");
            Err(e)
        },
        Ok(_) => {
            state.env().exception_describe().expect("this not to fail");
            state.env().exception_clear().expect("this not to fail");
            Ok(())
        },
    }?;

    let msg = state.msg();

    info!("handling websocket msg: '{msg}'...");
    let final_state = match msg {
        WebSocketMessagesEncodable::GetUserOps => super::handlers::get_user_ops(state),
        WebSocketMessagesEncodable::GetUserOpList => super::handlers::get_user_op_list(state),
        WebSocketMessagesEncodable::Initialize(args) => super::handlers::init(*args.clone(), state),
        WebSocketMessagesEncodable::GetUserOp(uid) => super::handlers::get_user_op(uid.clone(), state),
        WebSocketMessagesEncodable::Submit(args) => super::handlers::submit_blocks(*args.clone(), state),
        WebSocketMessagesEncodable::GetStatus(mcids) => super::handlers::get_status(mcids.clone(), state),
        WebSocketMessagesEncodable::ResetChain(args) => super::handlers::reset_chain(*args.clone(), state),
        WebSocketMessagesEncodable::RemoveUserOp(uid) => super::handlers::remove_user_op(uid.clone(), state),
        WebSocketMessagesEncodable::GetCoreState(mcids) => super::handlers::get_core_state(mcids.clone(), state),
        WebSocketMessagesEncodable::GetLatestBlockNumbers(mcids) => {
            super::handlers::get_latest_block_numbers(mcids.clone(), state)
        },
        WebSocketMessagesEncodable::GetUserOpCancellationSiganture(args) => {
            super::handlers::get_user_op_cancellation_signature(*args.clone(), state)
        },
        WebSocketMessagesEncodable::DbOps(WebSocketMessagesEncodableDbOps::Get(k)) => {
            super::handlers::get(k.clone(), state)
        },
        WebSocketMessagesEncodable::DbOps(WebSocketMessagesEncodableDbOps::Delete(k)) => {
            super::handlers::delete(k.clone(), state)
        },
        WebSocketMessagesEncodable::GetCancellableUserOps(args) => {
            super::handlers::get_cancellable_user_ops(*args.clone(), state)
        },
        WebSocketMessagesEncodable::DbOps(WebSocketMessagesEncodableDbOps::Put(k, v)) => {
            super::handlers::put(k.clone(), v.clone(), state)
        },
        m => Err(WebSocketMessagesError::Unhandled(m.to_string()).into()),
    }?;

    match final_state.db().end_transaction() {
        Err(e) => {
            error!("error ending db tx: {e}");
            final_state.env().exception_describe().expect("this not to fail");
            final_state.env().exception_clear().expect("this not to fail");
            Err(e)
        },
        Ok(_) => {
            final_state.env().exception_describe().expect("this not to fail");
            final_state.env().exception_clear().expect("this not to fail");
            Ok(())
        },
    }?;

    Ok(final_state)
}
