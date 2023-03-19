use std::result::Result;

use common::CoreType;
use lib::{CoreAccessorMessages, MongoAccessorMessages, SentinelConfig, SentinelError};
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

async fn main_loop(
    core_accessor_tx: MpscTx<CoreAccessorMessages>,
    mongo_accessor_tx: MpscTx<MongoAccessorMessages>,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    debug!("server listening!");

    // GET /
    let welcome = warp::path::end().map(|| "pTokens Sentinel is online!");

    // GET /state
    let state = warp::path("state").and_then(move || {
        let tx = core_accessor_tx.clone();
        let core_type = config.core_config.core_type;
        async move { get_core_state_from_db(tx, &core_type).await }
    });

    // GET /bpm
    let bpm = warp::path("bpm").and_then(move || {
        let tx = mongo_accessor_tx.clone();
        async move { get_heartbeat_from_db(tx).await }
    });

    let routes = warp::get().and(welcome.or(state).or(bpm));
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
