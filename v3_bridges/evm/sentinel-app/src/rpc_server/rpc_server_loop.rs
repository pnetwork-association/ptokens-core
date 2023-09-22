use common_sentinel::{
    BroadcastChannelMessages,
    RpcServerBroadcastChannelMessages,
    SentinelConfig,
    SentinelError,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use warp::{reject::Reject, Filter, Rejection};

use crate::rpc_server::constants::{
    BroadcastChannelRx,
    BroadcastChannelTx,
    BroadcasterTx,
    CoreCxnStatus,
    EthRpcTx,
    RpcId,
    RpcParams,
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

// FIXME make a type for error code
fn create_json_rpc_response_from_result<T: Serialize>(id: RpcId, r: Result<T, SentinelError>, error_code: u64) -> Json {
    match r {
        Ok(r) => create_json_rpc_response(id, r),
        Err(e) => create_json_rpc_error(id, error_code, &e.to_string()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    id: RpcId,
    method: String,
    params: RpcParams,
}

pub(crate) enum RpcCall {
    Ping(RpcId),
    Unknown(RpcId, String),
    GetUserOps(RpcId, WebSocketTx, CoreCxnStatus),
    GetCoreState(RpcId, WebSocketTx, CoreCxnStatus),
    GetUserOpList(RpcId, WebSocketTx, CoreCxnStatus),
    Get(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    Put(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    CancelUserOps(RpcId, BroadcasterTx, CoreCxnStatus),
    Delete(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    LatestBlockNumbers(RpcId, WebSocketTx, CoreCxnStatus),
    RemoveUserOp(RpcId, WebSocketTx, RpcParams, CoreCxnStatus),
    StopSyncer(RpcId, BroadcastChannelTx, RpcParams, CoreCxnStatus),
    StartSyncer(RpcId, BroadcastChannelTx, RpcParams, CoreCxnStatus),
    BroadcasterStartStop(RpcId, BroadcastChannelTx, CoreCxnStatus, bool),
    Init(RpcId, EthRpcTx, EthRpcTx, WebSocketTx, RpcParams, CoreCxnStatus),
    GetCancellableUserOps(RpcId, Box<SentinelConfig>, WebSocketTx, CoreCxnStatus),
    GetSyncState(
        RpcId,
        Box<SentinelConfig>,
        WebSocketTx,
        EthRpcTx,
        EthRpcTx,
        CoreCxnStatus,
    ),
    GetUserOpState(
        RpcId,
        Box<SentinelConfig>,
        WebSocketTx,
        EthRpcTx,
        EthRpcTx,
        RpcParams,
        CoreCxnStatus,
    ),
    ResetChain(
        RpcId,
        Box<SentinelConfig>,
        EthRpcTx,
        EthRpcTx,
        WebSocketTx,
        RpcParams,
        CoreCxnStatus,
    ),
    SubmitBlock(
        RpcId,
        Box<SentinelConfig>,
        EthRpcTx,
        EthRpcTx,
        WebSocketTx,
        RpcParams,
        bool,
    ),
}

#[allow(clippy::too_many_arguments)]
impl RpcCall {
    fn new(
        r: JsonRpcRequest,
        config: SentinelConfig,
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        broadcaster_tx: BroadcasterTx,
        broadcast_channel_tx: BroadcastChannelTx,
        core_cxn: bool,
    ) -> Self {
        match r.method.as_ref() {
            "ping" => Self::Ping(r.id),
            "getUserOps" => Self::GetUserOps(r.id, websocket_tx, core_cxn),
            "get" => Self::Get(r.id, websocket_tx, r.params.clone(), core_cxn),
            "put" => Self::Put(r.id, websocket_tx, r.params.clone(), core_cxn),
            "getUserOpList" => Self::GetUserOpList(r.id, websocket_tx, core_cxn),
            "delete" => Self::Delete(r.id, websocket_tx, r.params.clone(), core_cxn),
            "removeUserOp" => Self::RemoveUserOp(r.id, websocket_tx, r.params.clone(), core_cxn),
            "stopSyncer" => Self::StopSyncer(r.id, broadcast_channel_tx, r.params.clone(), core_cxn),
            "latestBlockNumbers" | "latest" => Self::LatestBlockNumbers(r.id, websocket_tx, core_cxn),
            "startSyncer" => Self::StartSyncer(r.id, broadcast_channel_tx, r.params.clone(), core_cxn),
            "startBroadcaster" => Self::BroadcasterStartStop(r.id, broadcast_channel_tx, core_cxn, true),
            "stopBroadcaster" => Self::BroadcasterStartStop(r.id, broadcast_channel_tx, core_cxn, false),
            "getCoreState" | "getEnclaveState" | "state" => Self::GetCoreState(r.id, websocket_tx, core_cxn),
            "cancel" | "cancelUserOp" => Self::CancelUserOps(r.id, broadcaster_tx.clone(), core_cxn),
            "getSyncState" => Self::GetSyncState(
                r.id,
                Box::new(config),
                websocket_tx,
                host_eth_rpc_tx,
                native_eth_rpc_tx,
                core_cxn,
            ),
            "getUserOpState" => Self::GetUserOpState(
                r.id,
                Box::new(config),
                websocket_tx,
                host_eth_rpc_tx,
                native_eth_rpc_tx,
                r.params.clone(),
                core_cxn,
            ),
            "getCancellableUserOps" | "getCancellable" => {
                Self::GetCancellableUserOps(r.id, Box::new(config), websocket_tx, core_cxn)
            },
            "reset" | "resetChain" => Self::ResetChain(
                r.id,
                Box::new(config),
                host_eth_rpc_tx,
                native_eth_rpc_tx,
                websocket_tx,
                r.params.clone(),
                core_cxn,
            ),
            "init" => Self::Init(
                r.id,
                host_eth_rpc_tx,
                native_eth_rpc_tx,
                websocket_tx,
                r.params.clone(),
                core_cxn,
            ),
            "submitBlock" | "submit" => Self::SubmitBlock(
                r.id,
                Box::new(config),
                host_eth_rpc_tx,
                native_eth_rpc_tx,
                websocket_tx,
                r.params.clone(),
                core_cxn,
            ),
            _ => Self::Unknown(r.id, r.method.clone()),
        }
    }

    pub(crate) fn check_core_is_connected(is_connected: bool) -> Result<(), SentinelError> {
        if is_connected {
            Ok(())
        } else {
            Err(SentinelError::NoCore)
        }
    }

    pub(crate) fn create_args(_cmd: &str, params: RpcParams) -> RpcParams {
        [vec!["init".to_string()], params].concat()
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

    async fn handle(self) -> Result<impl warp::Reply, Rejection> {
        match self {
            Self::GetSyncState(id, config, websocket_tx, host_eth_rpc_tx, native_eth_rpc_tx, core_cxn) => {
                Self::handle_ws_result(
                    id,
                    Self::handle_sync_state(*config, websocket_tx, host_eth_rpc_tx, native_eth_rpc_tx, core_cxn).await,
                )
            },
            Self::Get(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get(websocket_tx, params, core_cxn).await)
            },
            Self::CancelUserOps(id, broadcaster_tx, core_cxn) => {
                let result = Self::handle_cancel_user_ops(broadcaster_tx, core_cxn).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::BroadcasterStartStop(id, broadcast_channel_tx, core_cxn, start_broadcaster) => {
                let result =
                    Self::handle_broadcaster_start_stop(broadcast_channel_tx, core_cxn, start_broadcaster).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::Put(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_put(websocket_tx, params, core_cxn).await)
            },
            Self::Delete(id, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_delete(websocket_tx, params, core_cxn).await)
            },
            Self::ResetChain(id, config, host_eth_rpc_tx, native_eth_rpc_tx, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(
                    id,
                    Self::handle_reset_chain(
                        *config,
                        host_eth_rpc_tx,
                        native_eth_rpc_tx,
                        websocket_tx,
                        params,
                        core_cxn,
                    )
                    .await,
                )
            },
            Self::StopSyncer(id, broadcast_channel_tx, params, core_cxn) => {
                let result = Self::handle_syncer_start_stop(broadcast_channel_tx, params, true, core_cxn).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::GetUserOpState(id, config, websocket_tx, host_eth_rpc_tx, native_eth_rpc_tx, params, core_cxn) => {
                Self::handle_ws_result(
                    id,
                    Self::handle_get_user_op_state(
                        *config,
                        websocket_tx,
                        host_eth_rpc_tx,
                        native_eth_rpc_tx,
                        params,
                        core_cxn,
                    )
                    .await,
                )
            },
            Self::StartSyncer(id, broadcast_channel_tx, params, core_cxn) => {
                // TODO enum for syncer state
                let result = Self::handle_syncer_start_stop(broadcast_channel_tx, params, false, core_cxn).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::LatestBlockNumbers(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_latest_block_numbers(websocket_tx, core_cxn).await)
            },
            Self::SubmitBlock(id, config, host_eth_rpc_tx, native_eth_rpc_tx, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(
                    id,
                    Self::handle_submit_block(
                        *config,
                        host_eth_rpc_tx,
                        native_eth_rpc_tx,
                        websocket_tx,
                        params,
                        core_cxn,
                    )
                    .await,
                )
            },
            Self::Ping(id) => Ok(warp::reply::json(&create_json_rpc_response(id, "pong"))),
            Self::Init(id, host_eth_rpc_tx, native_eth_rpc_tx, websocket_tx, params, core_cxn) => {
                Self::handle_ws_result(
                    id,
                    Self::handle_init(websocket_tx, host_eth_rpc_tx, native_eth_rpc_tx, params, core_cxn).await,
                )
            },
            Self::GetCoreState(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_core_state(websocket_tx, core_cxn).await)
            },
            Self::GetUserOps(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_user_ops(websocket_tx, core_cxn).await)
            },
            Self::GetUserOpList(id, websocket_tx, core_cxn) => {
                Self::handle_ws_result(id, Self::handle_get_user_op_list(websocket_tx, core_cxn).await)
            },
            Self::GetCancellableUserOps(id, config, websocket_tx, core_cxn) => Self::handle_ws_result(
                id,
                Self::handle_get_cancellable_user_ops(config, websocket_tx, core_cxn).await,
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
        let j = match r {
            Ok(WebSocketMessagesEncodable::Success(j)) => create_json_rpc_response(id, j),
            other => create_json_rpc_response_from_result(id, other, 1337),
        };
        Ok(warp::reply::json(&j))
    }
}

async fn start_rpc_server(
    host_eth_rpc_tx: EthRpcTx,
    native_eth_rpc_tx: EthRpcTx,
    websocket_tx: WebSocketTx,
    config: SentinelConfig,
    broadcast_channel_tx: BroadcastChannelTx,
    core_cxn: bool,
    broadcaster_tx: BroadcasterTx,
) -> Result<(), SentinelError> {
    debug!("rpc server listening!");
    let core_cxn_filter = warp::any().map(move || core_cxn);
    let websocket_tx_filter = warp::any().map(move || websocket_tx.clone());
    let broadcaster_tx_filter = warp::any().map(move || broadcaster_tx.clone());
    let host_eth_rpc_tx_filter = warp::any().map(move || host_eth_rpc_tx.clone());
    let native_eth_rpc_tx_filter = warp::any().map(move || native_eth_rpc_tx.clone());
    let broadcast_channel_tx_filter = warp::any().map(move || broadcast_channel_tx.clone());

    let rpc = warp::path("v1")
        .and(warp::path("rpc"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16)) // FIXME make configurable
        .and(warp::body::json::<JsonRpcRequest>())
        .and(warp::any().map(move || config.clone()))
        .and(websocket_tx_filter.clone())
        .and(host_eth_rpc_tx_filter.clone())
        .and(native_eth_rpc_tx_filter.clone())
        .and(broadcaster_tx_filter.clone())
        .and(broadcast_channel_tx_filter.clone())
        .and(core_cxn_filter)
        .map(RpcCall::new)
        .and_then(|r: RpcCall| async move { r.handle().await });

    warp::serve(rpc).run(([127, 0, 0, 1], 3030)).await; // FIXME make configurable

    Ok(())
}

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: BroadcastChannelRx,
    _core_connection: bool,
) -> Result<RpcServerBroadcastChannelMessages, SentinelError> {
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::RpcServer(msg)) => {
                // NOTE: We have a pertinent message, break and send it to the rpc_server_loop...
                break 'broadcast_channel_loop Ok(msg);
            },
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for the syncer
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn rpc_server_loop(
    host_eth_rpc_tx: EthRpcTx,
    native_eth_rpc_tx: EthRpcTx,
    websocket_tx: WebSocketTx,
    config: SentinelConfig,
    disable: bool,
    broadcast_channel_tx: BroadcastChannelTx,
    broadcaster_tx: BroadcasterTx,
) -> Result<(), SentinelError> {
    let rpc_server_is_enabled = !disable;
    let name = "rpc server";
    if disable {
        warn!("{name} disabled!")
    };

    let mut core_connection_status = false;

    'rpc_server_loop: loop {
        tokio::select! {
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe(), core_connection_status) => {
                match r {
                    Ok(RpcServerBroadcastChannelMessages::CoreConnected) => {
                        core_connection_status = true;
                        continue 'rpc_server_loop
                    },
                    Ok(RpcServerBroadcastChannelMessages::CoreDisconnected) => {
                        core_connection_status = false;
                        continue 'rpc_server_loop
                    },
                    Err(e) => break 'rpc_server_loop Err(e),
                }
            },
            r = start_rpc_server(
                host_eth_rpc_tx.clone(),
                native_eth_rpc_tx.clone(),
                websocket_tx.clone(),
                config.clone(),
                broadcast_channel_tx.clone(),
                core_connection_status,
                broadcaster_tx.clone(),
            ), if rpc_server_is_enabled => {
                if r.is_ok() {
                    warn!("{name} returned, restarting {name} now...");
                    continue 'rpc_server_loop
                } else {
                    break 'rpc_server_loop r
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'rpc_server_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if rpc_server_is_enabled { "en" } else { "dis" });
                continue 'rpc_server_loop
            }
        }
    }
}
