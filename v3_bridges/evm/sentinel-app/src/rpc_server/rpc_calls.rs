use common_sentinel::{
    Env,
    EthRpcSenders,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use warp::{reject::Reject, Rejection};

use super::{
    type_aliases::{RpcId, RpcParams},
    JsonRpcRequest,
};
use crate::type_aliases::{
    BroadcastChannelTx,
    ChallengeResponderTx,
    CoreCxnStatus,
    StatusPublisherTx,
    UserOpCancellerTx,
    WebSocketTx,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error(String);

impl Reject for Error {}

fn create_json_rpc_response<T: Serialize>(id: RpcId, t: T) -> Json {
    json!({ "id": id, "result": t, "jsonrpc": "2.0" })
}

fn create_json_rpc_error(id: RpcId, code: u64, msg: &str) -> Json {
    json!({ "id": id, "error": { "code": code, "message": msg, }, "jsonrpc": "2.0" })
}

fn create_json_rpc_error_with_json<T: Serialize>(id: RpcId, code: u64, msg: &T) -> Json {
    json!({ "id": id, "error": { "code": code, "message": msg, }, "jsonrpc": "2.0" })
}

// FIXME make a type for error code
fn create_json_rpc_response_from_result<T: Serialize>(id: RpcId, r: Result<T, SentinelError>, error_code: u64) -> Json {
    match r {
        Ok(r) => create_json_rpc_response(id, r),
        Err(SentinelError::Json(ref j)) => create_json_rpc_error_with_json(id, error_code, j),
        Err(e) => create_json_rpc_error(id, error_code, &e.to_string()),
    }
}

pub(crate) enum RpcCalls {
    Ping(RpcId),
    Unknown(RpcId, String),
    SignMessage(RpcId, RpcParams),
    GetUserOps(RpcId, WebSocketTx, CoreCxnStatus),
    GetUserOpList(RpcId, WebSocketTx, CoreCxnStatus),
    Get(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    Put(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    GetInclusionProof(RpcId, WebSocketTx, CoreCxnStatus),
    GetChallengesList(RpcId, WebSocketTx, CoreCxnStatus),
    Delete(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    CancelUserOps(RpcId, UserOpCancellerTx, CoreCxnStatus),
    GetUserOp(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    GetStatus(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    HardReset(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    GetUnsolvedChallenges(RpcId, WebSocketTx, CoreCxnStatus),
    StatusPublisherStartStop(RpcId, BroadcastChannelTx, bool),
    RemoveUserOp(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    GetChallenge(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    GetCoreState(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    GetAttestionCertificate(RpcId, WebSocketTx, CoreCxnStatus),
    ChallengeResponderStartStop(RpcId, BroadcastChannelTx, bool),
    AddDebugSigners(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    RemoveChallenge(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    LatestBlockInfos(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    StopSyncer(RpcId, BroadcastChannelTx, RpcParams, CoreCxnStatus),
    StartSyncer(RpcId, BroadcastChannelTx, RpcParams, CoreCxnStatus),
    SetUserOpCancellerFrequency(RpcId, RpcParams, UserOpCancellerTx),
    GetBalances(RpcId, Box<SentinelConfig>, RpcParams, EthRpcSenders),
    SetStatusPublishingFrequency(RpcId, RpcParams, StatusPublisherTx),
    GetAttestionSignature(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    GetCancellableUserOps(RpcId, RpcParams, WebSocketTx, CoreCxnStatus),
    SetChallengeResponderFrequency(RpcId, RpcParams, ChallengeResponderTx),
    GetRegistrationSignature(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    UserOpCancellerStartStop(RpcId, BroadcastChannelTx, CoreCxnStatus, bool),
    GetRegistrationExtensionTx(RpcId, Box<SentinelConfig>, RpcParams, EthRpcSenders),
    GetChallengeState(
        RpcId,
        RpcParams,
        Box<SentinelConfig>,
        EthRpcSenders,
        WebSocketTx,
        CoreCxnStatus,
    ),
    Init(
        RpcId,
        Box<SentinelConfig>,
        EthRpcSenders,
        WebSocketTx,
        RpcParams,
        CoreCxnStatus,
    ),
    GetSyncState(RpcId, Box<SentinelConfig>, WebSocketTx, EthRpcSenders, CoreCxnStatus),
    GetUserOpState(
        RpcId,
        Box<SentinelConfig>,
        WebSocketTx,
        EthRpcSenders,
        RpcParams,
        CoreCxnStatus,
    ),
    ResetChain(
        RpcId,
        Box<SentinelConfig>,
        EthRpcSenders,
        WebSocketTx,
        RpcParams,
        CoreCxnStatus,
    ),
    ProcessBlock(RpcId, Box<SentinelConfig>, EthRpcSenders, WebSocketTx, RpcParams, bool),
}

impl RpcCalls {
    pub(super) fn new(
        r: JsonRpcRequest,
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        eth_rpc_senders: EthRpcSenders,
        user_op_canceller_tx: UserOpCancellerTx,
        broadcast_channel_tx: BroadcastChannelTx,
        status_tx: StatusPublisherTx,
        challenge_responder_tx: ChallengeResponderTx,
        core_cxn: bool,
    ) -> Self {
        match r.method().as_ref() {
            "ping" => Self::Ping(*r.id()),
            "get" => Self::Get(*r.id(), websocket_tx, r.params(), core_cxn),
            "put" => Self::Put(*r.id(), websocket_tx, r.params(), core_cxn),
            "signMessage" | "sign" => Self::SignMessage(*r.id(), r.params()),
            "getUserOps" => Self::GetUserOps(*r.id(), websocket_tx, core_cxn),
            "delete" => Self::Delete(*r.id(), websocket_tx, r.params(), core_cxn),
            "getUserOpList" => Self::GetUserOpList(*r.id(), websocket_tx, core_cxn),
            "getUserOp" => Self::GetUserOp(*r.id(), r.params(), websocket_tx, core_cxn),
            "getInclusionProof" => Self::GetInclusionProof(*r.id(), websocket_tx, core_cxn),
            "removeUserOp" => Self::RemoveUserOp(*r.id(), websocket_tx, r.params(), core_cxn),
            "getChallenge" => Self::GetChallenge(*r.id(), websocket_tx, r.params(), core_cxn),
            "hardReset" => Self::HardReset(*r.id(), r.params(), websocket_tx.clone(), core_cxn),
            "stopSyncer" => Self::StopSyncer(*r.id(), broadcast_channel_tx, r.params(), core_cxn),
            "getStatus" | "status" => Self::GetStatus(*r.id(), websocket_tx, r.params(), core_cxn),
            "startSyncer" => Self::StartSyncer(*r.id(), broadcast_channel_tx, r.params(), core_cxn),
            "getChallangeResponses" => Self::GetUnsolvedChallenges(*r.id(), websocket_tx, core_cxn),
            "getBalances" => Self::GetBalances(*r.id(), Box::new(config), r.params(), eth_rpc_senders),
            "getAttestationCertificate" => Self::GetAttestionCertificate(*r.id(), websocket_tx, core_cxn),
            "cancel" | "cancelUserOps" => Self::CancelUserOps(*r.id(), user_op_canceller_tx.clone(), core_cxn),
            "startChallengeResponder" => Self::ChallengeResponderStartStop(*r.id(), broadcast_channel_tx, true),
            "stopChallengeResponder" => Self::ChallengeResponderStartStop(*r.id(), broadcast_channel_tx, false),
            "getChallengesList" | "getChallengeList" => Self::GetChallengesList(*r.id(), websocket_tx, core_cxn),
            "setStatusPublishingFrequency" => Self::SetStatusPublishingFrequency(*r.id(), r.params(), status_tx),
            "getAttestationSignature" => Self::GetAttestionSignature(*r.id(), r.params(), websocket_tx, core_cxn),
            "removeChallenge" | "rmChallenge" => Self::RemoveChallenge(*r.id(), websocket_tx, r.params(), core_cxn),
            "addDebugSigners" | "addDebugSigner" => Self::AddDebugSigners(*r.id(), r.params(), websocket_tx, core_cxn),
            "getRegistrationExtensionTx" => {
                Self::GetRegistrationExtensionTx(*r.id(), Box::new(config.clone()), r.params(), eth_rpc_senders.clone())
            },
            "setUserOpCancellerFrequency" => {
                Self::SetUserOpCancellerFrequency(*r.id(), r.params(), user_op_canceller_tx)
            },
            "setChallengeResponderFrequency" => {
                Self::SetChallengeResponderFrequency(*r.id(), r.params(), challenge_responder_tx)
            },
            "stopUserOpCanceller" | "stopCanceller" => {
                Self::UserOpCancellerStartStop(*r.id(), broadcast_channel_tx, core_cxn, false)
            },
            "startUserOpCanceller" | "startCanceller" => {
                Self::UserOpCancellerStartStop(*r.id(), broadcast_channel_tx, core_cxn, true)
            },
            "getRegistrationSignature" | "getRegSig" => {
                Self::GetRegistrationSignature(*r.id(), websocket_tx, r.params(), core_cxn)
            },
            "stopStatusPublisher" | "stopPublisher" => {
                Self::StatusPublisherStartStop(*r.id(), broadcast_channel_tx.clone(), false)
            },
            "startStatusPublisher" | "startPublisher" => {
                Self::StatusPublisherStartStop(*r.id(), broadcast_channel_tx.clone(), true)
            },
            "getLatestBlockInfos" | "latest" => Self::LatestBlockInfos(*r.id(), r.params(), websocket_tx, core_cxn),
            "getCoreState" | "getEnclaveState" | "state" => {
                Self::GetCoreState(*r.id(), r.params(), websocket_tx, core_cxn)
            },
            "getChallengeState" => Self::GetChallengeState(
                *r.id(),
                r.params(),
                Box::new(config.clone()),
                eth_rpc_senders,
                websocket_tx,
                core_cxn,
            ),
            "getSyncState" => Self::GetSyncState(*r.id(), Box::new(config), websocket_tx, eth_rpc_senders, core_cxn),
            "getUserOpState" => Self::GetUserOpState(
                *r.id(),
                Box::new(config),
                websocket_tx,
                eth_rpc_senders,
                r.params(),
                core_cxn,
            ),
            "getCancellableUserOps" | "getCancellable" => {
                Self::GetCancellableUserOps(*r.id(), r.params(), websocket_tx, core_cxn)
            },
            "reset" | "resetChain" => Self::ResetChain(
                *r.id(),
                Box::new(config),
                eth_rpc_senders,
                websocket_tx,
                r.params(),
                core_cxn,
            ),
            "init" => Self::Init(
                *r.id(),
                Box::new(config),
                eth_rpc_senders,
                websocket_tx,
                r.params(),
                core_cxn,
            ),
            "processBlock" | "process" | "submitBlock" | "submit" => Self::ProcessBlock(
                *r.id(),
                Box::new(config),
                eth_rpc_senders,
                websocket_tx,
                r.params(),
                core_cxn,
            ),
            _ => Self::Unknown(*r.id(), r.method()),
        }
    }

    pub(crate) fn check_core_is_connected(is_connected: bool) -> Result<(), SentinelError> {
        if is_connected {
            Ok(())
        } else {
            Err(SentinelError::NoCore)
        }
    }

    pub(crate) fn create_args(cmd: &str, params: RpcParams) -> RpcParams {
        [vec![cmd.to_string()], params].concat()
    }

    pub(crate) fn check_params(
        params: RpcParams,
        required_num_params: usize,
    ) -> Result<RpcParams, WebSocketMessagesError> {
        if params.len() != required_num_params {
            Err(WebSocketMessagesError::NotEnoughArgs {
                got: params.len(),
                expected: required_num_params,
                args: params,
            })
        } else {
            Ok(params)
        }
    }

    pub(super) async fn handle(self) -> Result<impl warp::Reply, Rejection> {
        match self {
            Self::SetStatusPublishingFrequency(id, status_tx, params) => {
                let result = Self::handle_set_status_publishing_frequency(status_tx, params).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::SetUserOpCancellerFrequency(id, user_op_canceller_tx, params) => {
                let result = Self::handle_set_user_op_canceller_frequency(user_op_canceller_tx, params).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::SetChallengeResponderFrequency(id, challenge_tx, params) => {
                let result = Self::handle_set_challenge_responder_frequency(challenge_tx, params).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::GetSyncState(id, config, websocket_tx, eth_rpc_senders, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_sync_state(*config, websocket_tx, eth_rpc_senders, core_cxn).await,
            ),
            Self::AddDebugSigners(id, params, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_add_debug_signers(params, websocket_tx, core_cxn).await)
            },
            Self::HardReset(id, params, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_hard_reset(params, websocket_tx, core_cxn).await)
            },
            Self::GetRegistrationSignature(id, websocket_tx, params, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_registration_signature(websocket_tx, params, core_cxn).await,
            ),
            Self::GetRegistrationExtensionTx(id, config, params, eth_rpc_senders) => {
                let err_msg = "could not get private key from environment!";
                if let Err(e) = Env::init() {
                    error!("{e}");
                    return Ok(warp::reply::json(&create_json_rpc_error(id, 1337, err_msg)));
                };

                let pk = match Env::get_private_key() {
                    Ok(k) => k,
                    Err(e) => {
                        error!("{e}");
                        return Ok(warp::reply::json(&create_json_rpc_error(id, 1337, err_msg)));
                    },
                };

                let result = Self::handle_get_registration_extension_tx(*config, params, pk, eth_rpc_senders).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::Get(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get(websocket_tx, params, core_cxn).await)
            },
            Self::GetStatus(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_status(websocket_tx, params, core_cxn).await)
            },
            Self::CancelUserOps(id, user_op_canceller_tx, core_cxn) => {
                let result = Self::handle_cancel_user_ops(user_op_canceller_tx, core_cxn).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::UserOpCancellerStartStop(id, broadcast_channel_tx, core_cxn, start) => {
                let result = Self::handle_user_op_canceller_start_stop(broadcast_channel_tx, core_cxn, start).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::SignMessage(id, params) => {
                let result = Self::handle_sign_message(params).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::ChallengeResponderStartStop(id, broadcast_channel_tx, start) => {
                let result = Self::handle_challenge_responder_start_stop(broadcast_channel_tx, start).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::StatusPublisherStartStop(id, broadcast_channel_tx, start_status_publisher) => {
                let result =
                    Self::handle_status_publisher_start_stop(broadcast_channel_tx, start_status_publisher).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::Put(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_put(websocket_tx, params, core_cxn).await)
            },
            Self::Delete(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_delete(websocket_tx, params, core_cxn).await)
            },
            Self::ResetChain(id, config, eth_rpc_senders, websocket_tx, params, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_reset_chain(*config, eth_rpc_senders, websocket_tx, params, core_cxn).await,
            ),
            Self::StopSyncer(id, broadcast_channel_tx, params, core_cxn) => {
                let result = Self::handle_syncer_start_stop(broadcast_channel_tx, params, true, core_cxn).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::GetBalances(id, config, params, eth_rpc_senders) => {
                let result = Self::handle_get_balances(*config, params, eth_rpc_senders).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::GetUnsolvedChallenges(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_unsolved_challenges(websocket_tx, core_cxn).await)
            },
            Self::GetUserOpState(id, config, websocket_tx, eth_rpc_senders, params, core_cxn) => {
                Self::handle_ws_result(
                    id,
                    Self::handle_get_user_op_state(*config, websocket_tx, eth_rpc_senders, params, core_cxn).await,
                )
            },
            Self::GetChallengeState(id, params, config, eth_rpc_senders, websocket_tx, core_cxn) => {
                Self::handle_ws_result(
                    id,
                    Self::handle_get_challenge_state(*config, websocket_tx, eth_rpc_senders, params, core_cxn).await,
                )
            },
            Self::StartSyncer(id, broadcast_channel_tx, params, core_cxn) => {
                // TODO enum for syncer state
                let result = Self::handle_syncer_start_stop(broadcast_channel_tx, params, false, core_cxn).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::LatestBlockInfos(id, params, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_latest_block_infos(websocket_tx, params, core_cxn).await,
            ),
            Self::ProcessBlock(id, config, eth_rpc_senders, websocket_tx, params, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_process_block(*config, eth_rpc_senders, websocket_tx, params, core_cxn).await,
            ),
            Self::Ping(id) => Ok(warp::reply::json(&create_json_rpc_response(id, "pong"))),
            Self::Init(id, config, eth_rpc_senders, websocket_tx, params, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_init(*config, websocket_tx, eth_rpc_senders, params, core_cxn).await,
            ),
            Self::GetCoreState(id, params, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_core_state(params, websocket_tx, core_cxn).await)
            },
            Self::GetUserOps(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_user_ops(websocket_tx, core_cxn).await)
            },
            Self::GetAttestionCertificate(id, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_attestation_certificate(websocket_tx, core_cxn).await,
            ),
            Self::GetAttestionSignature(id, params, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_attestation_signature(websocket_tx, params, core_cxn).await,
            ),
            Self::GetUserOp(id, params, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_user_op(params, websocket_tx, core_cxn).await)
            },
            Self::GetUserOpList(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_user_op_list(websocket_tx, core_cxn).await)
            },
            Self::GetChallengesList(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_challenges_list(websocket_tx, core_cxn).await)
            },
            Self::GetChallenge(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_challenge(websocket_tx, params, core_cxn).await)
            },
            Self::RemoveChallenge(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_remove_challenge(websocket_tx, params, core_cxn).await)
            },
            Self::GetInclusionProof(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_inclusion_proof(websocket_tx, core_cxn).await)
            },
            Self::GetCancellableUserOps(id, params, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_cancellable_user_ops(websocket_tx, params, core_cxn).await,
            ),
            Self::RemoveUserOp(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_remove_user_op(websocket_tx, params, core_cxn).await)
            },
            Self::Unknown(id, method) => Ok(warp::reply::json(&create_json_rpc_error(
                id,
                1, // FIXME arbitrary
                &format!("unknown method: {method}"),
            ))),
        }
    }

    // NOTE: This is because anything involving the core returns an encodable result from which we
    // need to extract either the successful json response, or turn an error into an error response.
    fn handle_ws_result(
        id: RpcId,
        r: Result<WebSocketMessagesEncodable, SentinelError>,
    ) -> Result<warp::reply::Json, Rejection> {
        debug!("handling websocket encodable result: {r:?}");
        let error_code = 1337; // FIXME
        let j = match r {
            Ok(WebSocketMessagesEncodable::Success(j)) => create_json_rpc_response(id, j),
            Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::Json(ref j))) => {
                // NOTE: Otherwise we end up double stringifying the json
                create_json_rpc_error_with_json(id, error_code, j)
            },
            Ok(WebSocketMessagesEncodable::Error(e)) => {
                let s = e.to_string();
                // NOTE: We can't actually _get_ the exceptions from JNI on the core side of
                // things, we can only ask the JNI env to print them to console for us. So alas we
                // can't really do much. We can't even get a string to manually parse for common
                // errors etc. As such, this is literally the best we can do.
                let err_msg = if s.contains("Java exception was thrown") {
                    "a java exception was thrown - please see core logs for details".to_string()
                } else {
                    s
                };
                create_json_rpc_error(id, error_code, &err_msg)
            },
            other => create_json_rpc_response_from_result(id, other, error_code),
        };
        Ok(warp::reply::json(&j))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_rpc_error_messages_should_not_be_double_stringified() {
        let error_code = 666;
        let id = Some(1337);
        let j = json!({"some": "string"});
        let r = create_json_rpc_error(id, error_code, &j.to_string());
        let x = create_json_rpc_error_with_json(id, error_code, &j);
        assert!(r.to_string().contains("\\\""));
        assert!(!x.to_string().contains("\\\""));
    }
}
