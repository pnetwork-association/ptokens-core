use std::result::Result;

use common_sentinel::{
    SentinelError,
    WebSocketMessagesEncodable as MSG,
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
        MSG::GetUserOps => super::handlers::get_user_ops(state),
        MSG::GetUserOpList => super::handlers::get_user_op_list(state),
        MSG::Initialize(args) => super::handlers::init(*args.clone(), state),
        MSG::GetChallengesList => super::handlers::get_challenges_list(state),
        MSG::GetInclusionProof => super::handlers::get_inclusion_proof(state),
        MSG::GetChallenge(hash) => super::handlers::get_challenge(*hash, state),
        MSG::GetUserOp(uid) => super::handlers::get_user_op(uid.clone(), state),
        MSG::GetStatus(nids) => super::handlers::get_status(nids.clone(), state),
        MSG::ResetChain(args) => super::handlers::reset_chain(*args.clone(), state),
        MSG::GetUnsolvedChallenges => super::handlers::get_unsolved_challenges(state),
        MSG::RemoveChallenge(hash) => super::handlers::remove_challenge(*hash, state),
        MSG::CheckInit(network_id) => super::handlers::check_init(*network_id, state),
        MSG::RemoveUserOp(uid) => super::handlers::remove_user_op(uid.clone(), state),
        MSG::ProcessBatch(args) => super::handlers::process_batch(*args.clone(), state),
        MSG::GetCoreState(nids) => super::handlers::get_core_state(nids.clone(), state),
        MSG::GetAttestationCertificate => super::handlers::get_attestation_certificate(state),
        MSG::GetLatestBlockInfos(nids) => super::handlers::get_latest_block_infos(nids.clone(), state),
        MSG::SetChallengesToSolved(ids) => super::handlers::set_challenges_to_solved(ids.clone(), state),
        MSG::GetAttestationSignature(bytes) => super::handlers::get_attestation_signature(bytes.clone(), state),
        MSG::AddDebugSigners(signers, sig) => super::handlers::add_debug_signers(signers.clone(), sig.clone(), state),
        MSG::GetRegistrationSignature(owner, nonce, sig) => {
            super::handlers::get_registration_signature(*owner, *nonce, sig.clone(), state)
        },
        MSG::GetUserOpCancellationSignature(args) => {
            super::handlers::get_user_op_cancellation_signature(*args.clone(), state)
        },
        MSG::GetCancellableUserOps(args) => super::handlers::get_cancellable_user_ops(*args.clone(), state),
        MSG::DbOps(WebSocketMessagesEncodableDbOps::Get(k, sig)) => super::handlers::get(k.clone(), sig.clone(), state),
        MSG::DbOps(WebSocketMessagesEncodableDbOps::Delete(k, sig)) => {
            super::handlers::delete(k.clone(), sig.clone(), state)
        },
        MSG::DbOps(WebSocketMessagesEncodableDbOps::Put(k, v, sig)) => {
            super::handlers::put(k.clone(), v.clone(), sig.clone(), state)
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
