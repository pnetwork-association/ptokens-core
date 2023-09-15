use std::{str::FromStr, sync::Mutex};

use common::{BridgeSide, CoreType};
use common_chain_ids::EthChainId;
use common_eth::{convert_hex_to_h256, EthSubmissionMaterials};
use common_sentinel::{
    get_latest_block_num,
    BroadcastChannelMessages,
    CoreMessages,
    CoreState,
    EthRpcMessages,
    RpcServerBroadcastChannelMessages,
    SentinelConfig,
    SentinelError,
    SyncerBroadcastChannelMessages,
    UserOpList,
    UserOps,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesSubmitArgs,
};
use jsonrpsee::ws_client::WsClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use tokio::{
    sync::{
        broadcast::{Receiver as MpmcRx, Sender as MpmcTx},
        mpsc::Sender as MpscTx,
    },
    time::{sleep, Duration},
};
use warp::{reject, reject::Reject, Filter, Rejection};

type RpcId = Option<u64>;
type RpcParams = Vec<String>;
type CoreTx = MpscTx<CoreMessages>;
type EthRpcTx = MpscTx<EthRpcMessages>;
type WebSocketTx = MpscTx<WebSocketMessages>;
type BroadcastChannelTx = MpmcTx<BroadcastChannelMessages>;
type BroadcastChannelRx = MpmcRx<BroadcastChannelMessages>;

// TODO Need to re-instate BPM/HeartbeatsJson stuff, just kept in memory now rather than mongo

const STRONGBOX_TIMEOUT_MS: u64 = 30000; // FIXME make configurable

#[derive(Debug, Copy, Clone, Default)]
struct CoreConnectionStatus(bool);

impl CoreConnectionStatus {
    fn set_to_connected(mut self) {
        self.0 = true
    }

    fn set_to_disconnected(mut self) {
        self.0 = false
    }
}

lazy_static! {
    static ref CORE_CONNECTION_STATUS: Mutex<CoreConnectionStatus> = Mutex::new(CoreConnectionStatus::default());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error(String);

impl Reject for Error {}

fn convert_error_to_rejection<T: core::fmt::Display>(e: T) -> Rejection {
    reject::custom(Error(e.to_string())) // TODO rpc error spec adherence required
}

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

async fn get_core_state_from_db(tx: MpscTx<CoreMessages>, core_type: &CoreType) -> Result<CoreState, SentinelError> {
    let (msg, rx) = CoreMessages::get_core_state_msg(core_type);
    tx.send(msg).await?;
    rx.await?
}

async fn get_user_ops_from_core(tx: MpscTx<CoreMessages>) -> Result<UserOps, SentinelError> {
    let (msg, rx) = CoreMessages::get_user_ops_msg();
    tx.send(msg).await?;
    rx.await?
}

async fn get_user_ops_list_from_core(tx: MpscTx<CoreMessages>) -> Result<UserOpList, SentinelError> {
    let (msg, rx) = CoreMessages::get_user_ops_list_msg();
    tx.send(msg).await?;
    rx.await?
}

async fn remove_user_op_from_core(uid_string: String, tx: MpscTx<CoreMessages>) -> Result<Json, SentinelError> {
    let uid = convert_hex_to_h256(&uid_string)?;
    let (msg, rx) = CoreMessages::get_remove_user_op_msg(uid);
    tx.send(msg).await?;
    rx.await??;
    Ok(json!({"succes": true, "uid": format!("0x{}", hex::encode(uid.as_bytes()))}))
}

async fn get_sync_status(
    n_ws_client: &WsClient,
    h_ws_client: &WsClient,
    n_sleep_time: u64,
    h_sleep_time: u64,
    tx: MpscTx<CoreMessages>,
) -> Result<Json, SentinelError> {
    let n_e = get_latest_block_num(n_ws_client, n_sleep_time, BridgeSide::Native).await?;
    let h_e = get_latest_block_num(h_ws_client, h_sleep_time, BridgeSide::Host).await?;

    let (msg, rx) = CoreMessages::get_latest_block_numbers_msg();
    tx.send(msg).await?;
    let (n_c, h_c) = rx.await??;

    let n_d = if n_e > n_c { n_e - n_c } else { 0 };
    let h_d = if h_e > h_c { h_e - h_c } else { 0 };

    let r = json!({
        "host_delta": h_d,
        "native_delta": n_d,
        "host_core_latest_block_num": h_c,
        "native_core_latest_block_num": n_c,
        "host_endpoint_latest_block_num": h_e,
        "native_endpoint_latest_block_num": n_e,
    });

    Ok(r)
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    id: RpcId,
    method: String,
    params: RpcParams,
}

enum RpcCall {
    Ping(RpcId),
    Unknown(RpcId, String),
    GetUserOps(RpcId, CoreTx),
    GetUserOpList(RpcId, CoreTx),
    GetCoreState(RpcId, WebSocketTx),
    RemoveUserOp(RpcId, CoreTx, RpcParams),
    LatestBlockNumbers(RpcId, WebSocketTx),
    SyncStatus(RpcId, CoreTx, Box<SentinelConfig>),
    StopSyncer(RpcId, BroadcastChannelTx, RpcParams),
    StartSyncer(RpcId, BroadcastChannelTx, RpcParams),
    Init(RpcId, EthRpcTx, EthRpcTx, WebSocketTx, RpcParams),
    SubmitBlock(RpcId, Box<SentinelConfig>, EthRpcTx, EthRpcTx, WebSocketTx, RpcParams),
}

// TODO enum for error types with codes etc,then impl into for the rpc error type
impl RpcCall {
    fn new(
        r: JsonRpcRequest,
        config: SentinelConfig,
        core_tx: CoreTx,
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        broadcast_channel_tx: BroadcastChannelTx,
    ) -> Self {
        match r.method.as_ref() {
            "ping" => Self::Ping(r.id),
            "getUserOps" => Self::GetUserOps(r.id, core_tx),
            "getUserOpList" => Self::GetUserOpList(r.id, core_tx),
            "syncStatus" => Self::SyncStatus(r.id, core_tx, Box::new(config)),
            "removeUserOp" => Self::RemoveUserOp(r.id, core_tx, r.params.clone()),
            "stopSyncer" => Self::StopSyncer(r.id, broadcast_channel_tx, r.params.clone()),
            "latestBlockNumbers" | "latest" => Self::LatestBlockNumbers(r.id, websocket_tx),
            "startSyncer" => Self::StartSyncer(r.id, broadcast_channel_tx, r.params.clone()),
            "getCoreState" | "getEnclaveState" | "state" => Self::GetCoreState(r.id, websocket_tx),
            "init" => Self::Init(r.id, host_eth_rpc_tx, native_eth_rpc_tx, websocket_tx, r.params.clone()),
            "submitBlock" | "submit" => Self::SubmitBlock(
                r.id,
                Box::new(config),
                host_eth_rpc_tx,
                native_eth_rpc_tx,
                websocket_tx,
                r.params.clone(),
            ),
            _ => Self::Unknown(r.id, r.method.clone()),
        }
    }

    fn create_args(_cmd: &str, params: RpcParams) -> RpcParams {
        [vec!["init".to_string()], params].concat()
    }

    async fn handle_init(
        websocket_tx: WebSocketTx,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        params: RpcParams,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        check_core_is_connected()?;
        // NOTE: Get the latest host & native block numbers from the ETH RPC
        let (host_latest_block_num_msg, host_latest_block_num_responder) =
            EthRpcMessages::get_latest_block_num_msg(BridgeSide::Host);
        let (native_latest_block_num_msg, native_latest_block_num_responder) =
            EthRpcMessages::get_latest_block_num_msg(BridgeSide::Native);
        host_eth_rpc_tx.send(host_latest_block_num_msg).await?;
        native_eth_rpc_tx.send(native_latest_block_num_msg).await?;
        let host_latest_block_num = host_latest_block_num_responder.await??;
        let native_latest_block_num = native_latest_block_num_responder.await??;

        // NOTE: Get submission material for those latest block numbers
        let (host_latest_block_msg, host_latest_block_responder) =
            EthRpcMessages::get_sub_mat_msg(BridgeSide::Host, host_latest_block_num);
        let (native_latest_block_msg, native_latest_block_responder) =
            EthRpcMessages::get_sub_mat_msg(BridgeSide::Native, native_latest_block_num);
        host_eth_rpc_tx.send(host_latest_block_msg).await?;
        native_eth_rpc_tx.send(native_latest_block_msg).await?;
        let host_sub_mat = host_latest_block_responder.await??;
        let native_sub_mat = native_latest_block_responder.await??;

        let encodable_msg = WebSocketMessagesEncodable::try_from(Self::create_args("init", params))?;

        // NOTE: Now we need to add the sub mat to the args to send to strongbox
        let final_msg = match encodable_msg {
            WebSocketMessagesEncodable::Initialize(mut args) => {
                args.add_host_block(host_sub_mat);
                args.add_native_block(native_sub_mat);
                Ok(WebSocketMessagesEncodable::Initialize(args))
            },
            _ => Err(SentinelError::Custom("failed to crate initialize arguments".into())),
        }?;

        // NOTE: Now we send out msg to the websocket loop
        let (msg, rx) = WebSocketMessages::new(final_msg);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "initializing core";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }

    async fn handle_submit_block(
        config: SentinelConfig,
        host_eth_rpc_tx: EthRpcTx,
        native_eth_rpc_tx: EthRpcTx,
        websocket_tx: WebSocketTx,
        params: RpcParams,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        check_core_is_connected()?;
        let checked_params = Self::check_params(params, 4)?;

        let side = BridgeSide::from_str(&checked_params[0])?;
        let block_num = checked_params[1].parse::<u64>()?;
        let (eth_rpc_msg, responder) = EthRpcMessages::get_sub_mat_msg(side, block_num);
        if side.is_host() {
            host_eth_rpc_tx.send(eth_rpc_msg).await?;
        } else {
            native_eth_rpc_tx.send(eth_rpc_msg).await?;
        };
        let sub_mat = responder.await??;

        let dry_run = matches!(checked_params[2].as_ref(), "true");
        let reprocess = matches!(checked_params[3].as_ref(), "true");

        let submit_args = WebSocketMessagesSubmitArgs::new(
            dry_run,
            config.is_validating(&side),
            reprocess,
            side,
            config.chain_id(&side),
            config.pnetwork_hub(&side),
            EthSubmissionMaterials::new(vec![sub_mat]), // NOTE: The processor always deals with batches of submat
        );
        let encodable_msg = WebSocketMessagesEncodable::Submit(Box::new(submit_args));

        let (websocket_msg, rx) = WebSocketMessages::new(encodable_msg);
        websocket_tx.send(websocket_msg).await?;
        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "submitting block";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }

    fn check_params(params: RpcParams, required_num_params: usize) -> Result<RpcParams, WebSocketMessagesError> {
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

    async fn handle_get_core_state(websocket_tx: WebSocketTx) -> Result<WebSocketMessagesEncodable, SentinelError> {
        check_core_is_connected()?;
        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetCoreState);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "getting enclave state";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }

    async fn handle_get_latest_block_numbers(
        websocket_tx: WebSocketTx,
    ) -> Result<WebSocketMessagesEncodable, SentinelError> {
        check_core_is_connected()?;
        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetLatestBlockNumbers);
        websocket_tx.send(msg).await?;

        tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                let m = "getting latest block numbers";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }
    }

    async fn handle_syncer_start_stop(
        broadcast_channel_tx: BroadcastChannelTx,
        params: RpcParams,
        stop: bool,
    ) -> Result<Json, SentinelError> {
        debug!("handling stop syncer rpc call...");
        check_core_is_connected()?;
        let checked_params = Self::check_params(params, 1)?;
        let cid = EthChainId::from_str(&checked_params[0])?;
        let syncer_msg = if stop {
            SyncerBroadcastChannelMessages::Stop
        } else {
            SyncerBroadcastChannelMessages::Start
        };
        let m = if stop { "stop" } else { "start" };
        let json = json!({"status": format!("{m} message sent to {cid} syncer")});
        let broadcast_channel_msg = BroadcastChannelMessages::Syncer(cid, syncer_msg);
        broadcast_channel_tx.send(broadcast_channel_msg)?;
        Ok(json)
    }

    async fn handle(self) -> Result<impl warp::Reply, Rejection> {
        // TODO rm repetition in here.
        match self {
            Self::StopSyncer(id, broadcast_channel_tx, params) => {
                let result = Self::handle_syncer_start_stop(broadcast_channel_tx, params, true).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::StartSyncer(id, broadcast_channel_tx, params) => {
                let result = Self::handle_syncer_start_stop(broadcast_channel_tx, params, false).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::LatestBlockNumbers(id, websocket_tx) => {
                let result = Self::handle_get_latest_block_numbers(websocket_tx).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::SubmitBlock(id, config, host_eth_rpc_tx, native_eth_rpc_tx, websocket_tx, params) => {
                let result =
                    Self::handle_submit_block(*config, host_eth_rpc_tx, native_eth_rpc_tx, websocket_tx, params).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::Ping(id) => Ok(warp::reply::json(&create_json_rpc_response(id, "pong"))),
            Self::Init(id, host_eth_rpc_tx, native_eth_rpc_tx, websocket_tx, params) => {
                let result = Self::handle_init(websocket_tx, host_eth_rpc_tx, native_eth_rpc_tx, params).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::Unknown(id, method) => Ok(warp::reply::json(&create_json_rpc_error(
                id,
                1, // FIXME arbitrary
                &format!("unknown method: {method}"),
            ))),
            Self::GetCoreState(id, websocket_tx) => {
                let result = Self::handle_get_core_state(websocket_tx).await;
                let json = create_json_rpc_response_from_result(id, result, 1337);
                Ok(warp::reply::json(&json))
            },
            Self::GetUserOps(id, core_tx) => Ok(warp::reply::json(&create_json_rpc_response(
                id,
                get_user_ops_from_core(core_tx)
                    .await
                    .map_err(convert_error_to_rejection)?,
            ))),
            Self::GetUserOpList(id, core_tx) => Ok(warp::reply::json(&create_json_rpc_response(
                id,
                get_user_ops_list_from_core(core_tx)
                    .await
                    .map_err(convert_error_to_rejection)?,
            ))),
            Self::SyncStatus(id, core_tx, config) => {
                let h_endpoints = config.host().endpoints();
                let n_endpoints = config.native().endpoints();
                let h_sleep_time = h_endpoints.sleep_time();
                let n_sleep_time = n_endpoints.sleep_time();
                Ok(warp::reply::json(&create_json_rpc_response(
                    id,
                    get_sync_status(
                        &n_endpoints
                            .get_first_ws_client()
                            .await
                            .map_err(convert_error_to_rejection)?,
                        &h_endpoints
                            .get_first_ws_client()
                            .await
                            .map_err(convert_error_to_rejection)?,
                        n_sleep_time,
                        h_sleep_time,
                        core_tx,
                    )
                    .await
                    .map_err(convert_error_to_rejection)?,
                )))
            },
            Self::RemoveUserOp(id, core_tx, params) => {
                if params.is_empty() {
                    return Ok(warp::reply::json(&create_json_rpc_error(id, 1, "no params provided")));
                };

                Ok(warp::reply::json(&create_json_rpc_response(
                    id,
                    remove_user_op_from_core(params[0].clone(), core_tx)
                        .await
                        .map_err(convert_error_to_rejection)?,
                )))
            },
        }
    }
}

async fn start_rpc_server(
    core_tx: CoreTx,
    host_eth_rpc_tx: EthRpcTx,
    native_eth_rpc_tx: EthRpcTx,
    websocket_tx: WebSocketTx,
    config: SentinelConfig,
    broadcast_channel_tx: BroadcastChannelTx,
) -> Result<(), SentinelError> {
    debug!("rpc server listening!");
    let core_tx_filter = warp::any().map(move || core_tx.clone());
    let websocket_tx_filter = warp::any().map(move || websocket_tx.clone());
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
        .and(core_tx_filter.clone())
        .and(websocket_tx_filter.clone())
        .and(host_eth_rpc_tx_filter.clone())
        .and(native_eth_rpc_tx_filter.clone())
        .and(broadcast_channel_tx_filter.clone())
        .map(RpcCall::new)
        .and_then(|r: RpcCall| async move { r.handle().await });

    warp::serve(rpc).run(([127, 0, 0, 1], 3030)).await; // FIXME make configurable

    Ok(())
}

fn get_core_connection_status() -> Result<bool, SentinelError> {
    get_core_connection().map(|cxn| cxn.0)
}

fn set_core_connection_status(status: bool) -> Result<(), SentinelError> {
    let cxn = get_core_connection()?;
    if status {
        cxn.set_to_connected()
    } else {
        cxn.set_to_disconnected()
    };
    Ok(())
}

fn get_core_connection() -> Result<CoreConnectionStatus, SentinelError> {
    Ok(*CORE_CONNECTION_STATUS
        .lock()
        .map_err(|_| SentinelError::PoisonedLock("CORE_CONNECTION_STATUS in rpc server".into()))?)
}

fn check_core_is_connected() -> Result<(), SentinelError> {
    get_core_connection_status().and_then(|status| if status { Ok(()) } else { Err(SentinelError::NoCore) })
}

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: BroadcastChannelRx,
) -> Result<RpcServerBroadcastChannelMessages, SentinelError> {
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::RpcServer(msg)) => {
                // NOTE: We have a pertinent message...
                match msg {
                    RpcServerBroadcastChannelMessages::CoreConnected => {
                        // NOTE: We don't need to return here to handle this message, plus doing so
                        // would should down the server anyway which we don't need to do at this
                        // moment;
                        set_core_connection_status(true)?;
                        continue 'broadcast_channel_loop;
                    },
                    RpcServerBroadcastChannelMessages::CoreDisconnected => {
                        // NOTE: Ibid
                        set_core_connection_status(false)?;
                        continue 'broadcast_channel_loop;
                    },
                }
            },
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for the syncer
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn rpc_server_loop(
    core_tx: CoreTx,
    host_eth_rpc_tx: EthRpcTx,
    native_eth_rpc_tx: EthRpcTx,
    websocket_tx: WebSocketTx,
    config: SentinelConfig,
    disable: bool,
    broadcast_channel_tx: BroadcastChannelTx,
) -> Result<(), SentinelError> {
    let rpc_server_is_enabled = !disable;
    let name = "rpc server";
    if disable {
        warn!("{name} disabled!")
    };
    'rpc_server_loop: loop {
        tokio::select! {
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe()) => {
                // NOTE: Currently there are no messages from the broadcast channel that we'd need
                // to handle here, so we just continue the loop in case of returns
                debug!("brodacast channel loop in rpc server returned: {r:?}");
                continue 'rpc_server_loop
            },
            r = start_rpc_server(
                core_tx.clone(),
                host_eth_rpc_tx.clone(),
                native_eth_rpc_tx.clone(),
                websocket_tx.clone(),
                config.clone(),
                broadcast_channel_tx.clone(),
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
