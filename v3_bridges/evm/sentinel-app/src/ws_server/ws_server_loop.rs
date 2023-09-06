use std::{net::SocketAddr, ops::ControlFlow, path::PathBuf, result::Result, sync::Arc};

use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
    TypedHeader,
};
use common_sentinel::{CoreMessages, SentinelConfig, SentinelError, WebSocketMessages, WebSocketMessagesEncodable};
use derive_more::Constructor;
use futures::{stream::StreamExt, SinkExt};
use tokio::sync::{
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};
use tower_http::services::ServeDir;

// TODO have somewhere hold all these aliases
type CoreTx = MpscTx<CoreMessages>;
type WebSocketRx = MpscRx<WebSocketMessages>;

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            debug!("{} sent str: {:?}", who, t);
        },
        Message::Binary(d) => {
            debug!("{} sent {} bytes: {:?}", who, d.len(), d);
        },
        Message::Close(c) => {
            if let Some(cf) = c {
                debug!("{} sent close with code {} and reason `{}`", who, cf.code, cf.reason);
            } else {
                debug!("{} somehow sent close message without CloseFrame", who);
            }
            return ControlFlow::Break(());
        },

        Message::Pong(v) => {
            debug!("{} sent pong with {:?}", who, v);
        },
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            debug!("{} sent ping with {:?}", who, v);
        },
    }
    ControlFlow::Continue(())
}

async fn handle_socket(
    mut socket: WebSocket,
    who: SocketAddr,
    core_tx: CoreTx,
    websocket_rx: Arc<Mutex<WebSocketRx>>,
) -> Result<(), SentinelError> {
    if socket.send(Message::Ping(vec![1, 3, 3, 7])).await.is_ok() {
        debug!("pinged {}...", who);
    } else {
        error!("could not send ping {}!", who);
        // NOTE: No Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return Ok(()); // FIXME
    }

    // NOTE: Receive single message from a client (we can either receive or send with socket).
    // this will likely be the pong for our ping, or a hello message from client.
    // Waiting for a message from a client will block this task, but will not block other client's
    // connections.
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, who).is_break() {
                return Ok(());
            }
        } else {
            error!("client {who} abruptly disconnected");
            return Ok(()); // FIXME
        }
    }
    let (mut sender, mut receiver) = socket.split();

    // NOTE: Trying the lock here limits us to one active connection.
    let mut rx = websocket_rx.try_lock()?;

    'ws_loop: loop {
        tokio::select! {
            r = rx.recv() => {
                if let Some(WebSocketMessages(msg, responder)) = r {
                    // NOTE: Pass the message on to whomever is connected to the server.
                    sender.send(Message::Text(msg.try_into()?)).await?;

                    // TODO race a time out with tokio::select!
                    match receiver.next().await { // NOTE: Await a response since we always expect one
                        Some(Ok(Message::Text(m))) => {
                            // NOTE: Return the response to the whomever sent original msg
                            let _ = responder.send(WebSocketMessagesEncodable::try_from(m));
                            continue 'ws_loop
                        },
                        r => {
                            error!("websocket did not return with expected response: {r:?}");
                            break 'ws_loop
                        }
                    }
                } else {
                    error!("all websocket senders dropped");
                    break 'ws_loop
                }
            },
        }
    }

    error!("websocket context {who} destroyed");
    Ok(()) // FIXME Too many connections error or something?
}

#[derive(Clone)]
struct AppState {
    core_tx: CoreTx,
    websocket_rx: Arc<Mutex<WebSocketRx>>,
}

impl AppState {
    fn new(core_tx: CoreTx, websocket_rx: WebSocketRx) -> Self {
        Self {
            core_tx,
            websocket_rx: Arc::new(Mutex::new(websocket_rx)),
        }
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("unknown browser")
    };
    debug!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| async move {
        match handle_socket(socket, addr.clone(), state.core_tx, state.websocket_rx).await {
            // FIXME what to return from here?
            Ok(_) => (),
            Err(e) => {
                error!("websocket error: {e}");
            },
        }
    })
}

async fn start_ws_server(
    websocket_rx: WebSocketRx,
    core_tx: CoreTx,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin/sentinel/ws_server/assets");

    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        .with_state(AppState::new(core_tx, websocket_rx));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // FIXME make configurable
    debug!("ws server listening on {}", addr);

    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?)
}

pub async fn ws_server_loop(
    websocket_rx: WebSocketRx,
    core_tx: CoreTx,
    config: SentinelConfig,
    disable: bool,
) -> Result<(), SentinelError> {
    let name = "ws server";
    if disable {
        warn!("{name} disabled!")
    } else {
        debug!("{name} started")
    };
    let mut ws_server_is_enabled = !disable;

    tokio::select! {
        r = start_ws_server(websocket_rx, core_tx.clone(), config.clone()), if ws_server_is_enabled => r,
        _ = tokio::signal::ctrl_c() => {
            warn!("{name} shutting down...");
            Err(SentinelError::SigInt(name.into()))
        },
        else => {
            let m = format!("in {name} `else` branch, {name} is currently {}abled", if ws_server_is_enabled { "en" } else { "dis" });
            warn!("{m}");
            Err(SentinelError::Custom(m))
        }
    }
}
