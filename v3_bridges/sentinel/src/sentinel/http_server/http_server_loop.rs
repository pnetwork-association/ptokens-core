use std::result::Result;

use common::CoreType;
use lib::{
    get_latest_block_num,
    CoreAccessorMessages,
    Endpoints,
    MongoAccessorMessages,
    SentinelConfig,
    SentinelError,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender as MpscTx;
use warp::{reject, reject::Reject, Filter, Rejection};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error(String);

impl Reject for Error {}

fn convert_error_to_rejection<T: core::fmt::Display>(e: T) -> Rejection {
    reject::custom(Error(e.to_string()))
}

async fn get_core_state_from_db(
    tx: MpscTx<CoreAccessorMessages>,
    core_type: &CoreType,
) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = CoreAccessorMessages::get_core_state_msg(core_type);
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|core_state| warp::reply::json(&core_state))
}

async fn get_heartbeat_from_db(tx: MpscTx<MongoAccessorMessages>) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = MongoAccessorMessages::get_heartbeats_msg();
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|h| warp::reply::json(&h.to_output()))
}

async fn get_sync_status(
    n_endpoints: &Endpoints,
    h_endpoints: &Endpoints,
    tx: MpscTx<CoreAccessorMessages>,
) -> Result<impl warp::Reply, Rejection> {
    let n_endpoint = n_endpoints.get_rpc_client().await.map_err(convert_error_to_rejection)?;
    let h_endpoint = h_endpoints.get_rpc_client().await.map_err(convert_error_to_rejection)?;

    let n_e = get_latest_block_num(&n_endpoint)
        .await
        .map_err(convert_error_to_rejection)?;
    let h_e = get_latest_block_num(&h_endpoint)
        .await
        .map_err(convert_error_to_rejection)?;

    let (msg, rx) = CoreAccessorMessages::get_latest_block_numbers_msg();
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|(n_c, h_c)| {
            let n_d = if n_e > n_c { n_e - n_c } else { 0 };
            let h_d = if h_e > h_c { h_e - h_c } else { 0 };
            warp::reply::json(&serde_json::json!({
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
    core_accessor_tx: MpscTx<CoreAccessorMessages>,
    mongo_accessor_tx: MpscTx<MongoAccessorMessages>,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    debug!("server listening!");
    let core_tx_1 = core_accessor_tx.clone();
    let core_tx_2 = core_accessor_tx.clone();

    // GET /
    let welcome = warp::path::end().map(|| "pTokens Sentinel is online!");

    // GET /state
    let state = warp::path("state").and_then(move || {
        let tx = core_tx_1.clone();
        let core_type = config.core_config.core_type;
        async move { get_core_state_from_db(tx, &core_type).await }
    });

    // GET /bpm
    let bpm = warp::path("bpm").and_then(move || {
        let tx = mongo_accessor_tx.clone();
        async move { get_heartbeat_from_db(tx).await }
    });

    // GET /sync
    let sync = warp::path("sync").and_then(move || {
        let tx = core_tx_2.clone();
        let h_endpoints = config.host_config.get_endpoints();
        let n_endpoints = config.native_config.get_endpoints();
        async move { get_sync_status(&n_endpoints, &h_endpoints, tx).await }
    });

    let routes = warp::get().and(welcome.or(state).or(bpm).or(sync));
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

pub async fn http_server_loop(
    core_accessor_tx: MpscTx<CoreAccessorMessages>,
    mongo_accessor_tx: MpscTx<MongoAccessorMessages>,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    tokio::select! {
        _ = main_loop(core_accessor_tx, mongo_accessor_tx, config.clone()) => Ok(()),
        _ = tokio::signal::ctrl_c() => {
            warn!("http server shutting down...");
            Err(SentinelError::SigInt("http server".into()))
        },
    }
}
