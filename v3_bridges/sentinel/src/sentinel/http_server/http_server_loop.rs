use std::result::Result;

use common::{BridgeSide, CoreType};
use jsonrpsee::ws_client::WsClient;
use lib::{get_latest_block_num, CoreMessages, MongoMessages, SentinelConfig, SentinelError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use tokio::sync::mpsc::Sender as MpscTx;
use warp::{reject, reject::Reject, Filter, Rejection};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error(String);

impl Reject for Error {}

fn convert_error_to_rejection<T: core::fmt::Display>(e: T) -> Rejection {
    reject::custom(Error(e.to_string()))
}

// TODO rm duplicate code from here
async fn get_core_state_from_db(tx: MpscTx<CoreMessages>, core_type: &CoreType) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = CoreMessages::get_core_state_msg(core_type);
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|core_state| warp::reply::json(&core_state))
}

async fn get_user_ops_from_core(tx: MpscTx<CoreMessages>) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = CoreMessages::get_user_ops_msg();
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|core_state| warp::reply::json(&core_state))
}

async fn get_heartbeat_from_mongo(tx: MpscTx<MongoMessages>) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = MongoMessages::get_heartbeats_msg();
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|h| warp::reply::json(&h.to_output()))
}

async fn get_sync_status(
    n_ws_client: &WsClient,
    h_ws_client: &WsClient,
    n_sleep_time: u64,
    h_sleep_time: u64,
    tx: MpscTx<CoreMessages>,
) -> Result<impl warp::Reply, Rejection> {
    let n_e = get_latest_block_num(n_ws_client, n_sleep_time, BridgeSide::Native)
        .await
        .map_err(convert_error_to_rejection)?;
    let h_e = get_latest_block_num(h_ws_client, h_sleep_time, BridgeSide::Host)
        .await
        .map_err(convert_error_to_rejection)?;

    let (msg, rx) = CoreMessages::get_latest_block_numbers_msg();
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|(n_c, h_c)| {
            let n_d = if n_e > n_c { n_e - n_c } else { 0 };
            let h_d = if h_e > h_c { h_e - h_c } else { 0 };
            warp::reply::json(&json!({
                "host_delta": h_d,
                "native_delta": n_d,
                "host_core_latest_block_num": h_c,
                "native_core_latest_block_num": n_c,
                "host_endpoint_latest_block_num": h_e,
                "native_endpoint_latest_block_num": n_e,
            }))
        })
}

async fn main_loop(
    core_tx: MpscTx<CoreMessages>,
    mongo_tx: MpscTx<MongoMessages>,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    debug!("server listening!");
    let core_tx_1 = core_tx.clone();
    let core_tx_2 = core_tx.clone();
    let core_tx_3 = core_tx.clone();
    let mongo_tx_1 = mongo_tx.clone();
    let core_type = config.core().core_type;

    let h_endpoints = config.host().endpoints();
    let n_endpoints = config.native().endpoints();

    let h_sleep_time = h_endpoints.sleep_time();
    let n_sleep_time = n_endpoints.sleep_time();

    // GET /ping
    let ping = warp::path("ping").map(|| warp::reply::json(&json!({"result": "pTokens Sentinel pong"})));

    // GET /state
    let state = warp::path("state").and_then(move || {
        let tx = core_tx_1.clone();
        async move { get_core_state_from_db(tx, &core_type).await }
    });

    // GET /bpm
    let bpm = warp::path("bpm").and_then(move || {
        let tx = mongo_tx_1.clone();
        async move { get_heartbeat_from_mongo(tx).await }
    });

    // GET /sync
    let sync = warp::path("sync").and_then(move || {
        let tx = core_tx_2.clone();
        let he = h_endpoints.clone();
        let ne = n_endpoints.clone();
        async move {
            fn get_err(side: BridgeSide) -> JsonValue {
                json!({ "jsonrpc": "2.0", "error": format!("error getting {side} websocket - check your config")})
            }

            let h_ws_client = he.get_first_ws_client().await;
            let n_ws_client = ne.get_first_ws_client().await;

            if h_ws_client.is_err() {
                Err(reject::custom(Error(get_err(BridgeSide::Host).to_string())))
            } else if n_ws_client.is_err() {
                Err(reject::custom(Error(get_err(BridgeSide::Native).to_string())))
            } else {
                get_sync_status(
                    &n_ws_client.unwrap(),
                    &h_ws_client.unwrap(),
                    n_sleep_time,
                    h_sleep_time,
                    tx,
                )
                .await
            }
        }
    });

    // GET /ops
    let ops = warp::path("ops").and_then(move || {
        let tx = core_tx_3.clone();
        async move { get_user_ops_from_core(tx).await }
    });

    let routes = warp::get().and(ping.or(state).or(bpm).or(sync).or(ops));
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
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
