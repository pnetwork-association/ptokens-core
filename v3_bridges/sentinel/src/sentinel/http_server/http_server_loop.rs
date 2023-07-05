use common::{BridgeSide, CoreType};
use common_eth::convert_hex_to_h256;
use jsonrpsee::ws_client::WsClient;
use lib::{
    get_latest_block_num,
    CoreMessages,
    CoreState,
    HeartbeatsJson,
    MongoMessages,
    SentinelConfig,
    SentinelError,
    UserOpList,
    UserOps,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use tokio::sync::mpsc::Sender as MpscTx;
use warp::{reject, reject::Reject, Filter, Rejection};

type RpcId = Option<u64>;
type RpcParams = Vec<String>;
type CoreTx = MpscTx<CoreMessages>;
type MongoTx = MpscTx<MongoMessages>;

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

async fn get_heartbeat_from_mongo(tx: MongoTx) -> Result<HeartbeatsJson, SentinelError> {
    let (msg, rx) = MongoMessages::get_heartbeats_msg();
    tx.send(msg).await?;
    rx.await?
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
    params: Vec<String>,
}

enum RpcCall {
    Ping(RpcId),
    Bpm(RpcId, MongoTx),
    Unknown(RpcId, String),
    GetUserOps(RpcId, CoreTx),
    GetUserOpList(RpcId, CoreTx),
    GetCoreState(RpcId, CoreTx, CoreType),
    RemoveUserOp(RpcId, CoreTx, RpcParams),
    SyncStatus(RpcId, CoreTx, Box<SentinelConfig>),
}

// TODO enum for error types with codes etc,then impl into for the rpc error type
impl RpcCall {
    fn new(r: JsonRpcRequest, config: SentinelConfig, core_tx: CoreTx, mongo_tx: MongoTx) -> Self {
        match r.method.as_ref() {
            "ping" => Self::Ping(r.id),
            "bpm" => Self::Bpm(r.id, mongo_tx),
            "getUserOps" => Self::GetUserOps(r.id, core_tx),
            "getUserOpList" => Self::GetUserOpList(r.id, core_tx),
            "syncStatus" => Self::SyncStatus(r.id, core_tx, Box::new(config)),
            "removeUserOp" => Self::RemoveUserOp(r.id, core_tx, r.params.clone()),
            "getCoreState" => Self::GetCoreState(r.id, core_tx, config.core().core_type()),
            _ => Self::Unknown(r.id, r.method.clone()),
        }
    }

    async fn handle(self) -> Result<impl warp::Reply, Rejection> {
        match self {
            Self::Ping(id) => Ok(warp::reply::json(&create_json_rpc_response(id, "pong"))),
            Self::Unknown(id, method) => Ok(warp::reply::json(&create_json_rpc_error(
                id,
                1, // FIXME arbitrary
                &format!("unknown method: {method}"),
            ))),
            Self::GetCoreState(id, core_tx, core_type) => Ok(warp::reply::json(&create_json_rpc_response(
                id,
                get_core_state_from_db(core_tx, &core_type)
                    .await
                    .map_err(convert_error_to_rejection)?,
            ))),
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
            Self::Bpm(id, mongo_tx) => Ok(warp::reply::json(&create_json_rpc_response(
                id,
                get_heartbeat_from_mongo(mongo_tx)
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

async fn main_loop(core_tx: CoreTx, mongo_tx: MongoTx, config: SentinelConfig) -> Result<(), SentinelError> {
    debug!("server listening!");
    let core_tx_filter = warp::any().map(move || core_tx.clone());
    let mongo_tx_filter = warp::any().map(move || mongo_tx.clone());

    let rpc = warp::path("v1")
        .and(warp::path("rpc"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16)) // FIXME make configurable
        .and(warp::body::json::<JsonRpcRequest>())
        .and(warp::any().map(move || config.clone()))
        .and(core_tx_filter.clone())
        .and(mongo_tx_filter.clone())
        .map(RpcCall::new)
        .and_then(|r: RpcCall| async move { r.handle().await });

    warp::serve(rpc).run(([127, 0, 0, 1], 3030)).await; // FIXME make configurable

    Ok(())
}

pub async fn http_server_loop(
    core_tx: MpscTx<CoreMessages>,
    mongo_tx: MpscTx<MongoMessages>,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    tokio::select! {
        _ = main_loop(core_tx, mongo_tx, config.clone()) => Ok(()),
        _ = tokio::signal::ctrl_c() => {
            warn!("http server shutting down...");
            Err(SentinelError::SigInt("http server".into()))
        },
    }
}
