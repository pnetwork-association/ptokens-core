use std::result::Result;

use common::CoreType;
use lib::{get_latest_block_num, CoreMessages, Endpoints, MongoMessages, SentinelConfig, SentinelError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc::Sender as MpscTx;
use warp::{reject, reject::Reject, Filter, Rejection};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error(String);

impl Reject for Error {}

fn convert_error_to_rejection<T: core::fmt::Display>(e: T) -> Rejection {
    reject::custom(Error(e.to_string()))
}

async fn get_core_state_from_db(tx: MpscTx<CoreMessages>, core_type: &CoreType) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = CoreMessages::get_core_state_msg(core_type);
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|core_state| warp::reply::json(&core_state))
}

async fn get_unmatched_user_ops_from_db(tx: MpscTx<CoreMessages>) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = CoreMessages::get_unmatched_user_ops_msg();
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|r| warp::reply::json(&r))
}

async fn get_heartbeat_from_mongo(tx: MpscTx<MongoMessages>) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = MongoMessages::get_heartbeats_msg();
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|h| warp::reply::json(&h.to_output()))
}

async fn get_output_from_mongo(tx: MpscTx<MongoMessages>) -> Result<impl warp::Reply, Rejection> {
    let (msg, rx) = MongoMessages::get_output_msg();
    tx.send(msg).await.map_err(convert_error_to_rejection)?;
    rx.await
        .map_err(convert_error_to_rejection)?
        .map_err(convert_error_to_rejection)
        .map(|o| warp::reply::json(&o))
}

async fn get_sync_status(
    n_endpoints: &Endpoints,
    h_endpoints: &Endpoints,
    tx: MpscTx<CoreMessages>,
) -> Result<impl warp::Reply, Rejection> {
    let n_e = get_latest_block_num(n_endpoints)
        .await
        .map_err(convert_error_to_rejection)?;
    let h_e = get_latest_block_num(h_endpoints)
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
    let mongo_tx_2 = mongo_tx.clone();

    // GET /ping
    let ping = warp::path("ping").map(|| warp::reply::json(&json!({"result": "pTokens Sentinel pong"})));

    // GET /state
    let state = warp::path("state").and_then(move || {
        let tx = core_tx_1.clone();
        let core_type = config.core_config.core_type;
        #[allow(clippy::redundant_async_block)]
        async move {
            get_core_state_from_db(tx, &core_type).await
        }
    });

    // GET /bpm
    let bpm = warp::path("bpm").and_then(move || {
        let tx = mongo_tx_1.clone();
        #[allow(clippy::redundant_async_block)]
        async move {
            get_heartbeat_from_mongo(tx).await
        }
    });

    // GET /sync
    let sync = warp::path("sync").and_then(move || {
        let tx = core_tx_2.clone();
        let h_endpoints = config.host_config.get_endpoints();
        let n_endpoints = config.native_config.get_endpoints();
        #[allow(clippy::redundant_async_block)]
        async move {
            get_sync_status(&n_endpoints, &h_endpoints, tx).await
        }
    });

    // GET /unmatched
    let unmatched = warp::path("unmatched").and_then(move || {
        let tx = core_tx_3.clone();
        #[allow(clippy::redundant_async_block)]
        async move {
            get_unmatched_user_ops_from_db(tx).await
        }
    });

    // GET /output
    let output = warp::path("output").and_then(move || {
        let tx = mongo_tx_2.clone();
        #[allow(clippy::redundant_async_block)]
        async move {
            get_output_from_mongo(tx).await
        }
    });

    let routes = warp::get().and(ping.or(state).or(bpm).or(sync).or(unmatched).or(output));
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
