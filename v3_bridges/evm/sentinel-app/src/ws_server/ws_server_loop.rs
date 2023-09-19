use std::{net::SocketAddr, ops::ControlFlow, path::PathBuf, result::Result, sync::Arc, time::Duration};

use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
    TypedHeader,
};
use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_sentinel::{
    BroadcastChannelMessages,
    RpcServerBroadcastChannelMessages,
    SentinelConfig,
    SentinelError,
    SyncerBroadcastChannelMessages,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use futures::{stream::StreamExt, SinkExt};
use tokio::{
    sync::{broadcast::Sender as MpMcTx, mpsc::Receiver as MpscRx, Mutex},
    time::sleep,
};
use tower_http::services::ServeDir;

// TODO have somewhere hold all these aliases
type WebSocketRx = MpscRx<WebSocketMessages>;
type BroadcastChannelTx = MpMcTx<BroadcastChannelMessages>;

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
    websocket_rx: Arc<Mutex<WebSocketRx>>,
    broadcast_channel_tx: BroadcastChannelTx,
    cids: Vec<EthChainId>,
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

    for cid in cids.iter().cloned() {
        // NOTE: Tell the various components that a core is connected.
        broadcast_channel_tx.send(BroadcastChannelMessages::Syncer(
            cid,
            SyncerBroadcastChannelMessages::CoreConnected,
        ))?;
        broadcast_channel_tx.send(BroadcastChannelMessages::RpcServer(
            RpcServerBroadcastChannelMessages::CoreConnected,
        ))?;
    }

    'ws_loop: loop {
        tokio::select! {
            r = rx.recv() => {
                if let Some(WebSocketMessages(msg, responder)) = r {
                    // NOTE: Pass the message on to whomever is connected to the server.
                    sender.send(Message::Text(msg.try_into()?)).await?;

                    const STRONGBOX_TIMEOUT_MS: u64 = 30000; // TODO make configurable
                     // NOTE: We race the response against a timeout
                    tokio::select! {
                        _ = sleep(Duration::from_millis(STRONGBOX_TIMEOUT_MS)) => {
                            let response = WebSocketMessagesError::Timedout(STRONGBOX_TIMEOUT_MS);
                            let _ = responder.send(Err(response.into()));
                            continue 'ws_loop
                        },
                        r = receiver.next() => {
                            match r {
                                Some(Ok(Message::Text(m))) => {
                                    let _ = responder.send(WebSocketMessagesEncodable::try_from(m));
                                    continue 'ws_loop
                                },
                                r => {
                                    error!("websocket did not return with expected response: {r:?}");
                                    break 'ws_loop
                                }
                            }
                        },
                    }
                } else {
                    error!("all websocket senders dropped");
                    break 'ws_loop
                }
            },
            msg = receiver.next() => {
                // NOTE: When we send a message via the arm above this one, we await a response in
                // that block of code. As such, if this arm is ever tripped, we must have received
                // a message that we weren't prepared for, and we handle it thusly here.
                match msg {
                    Some(Ok(Message::Close(maybe_reason))) => {
                        warn!("close msg received from websocket");
                        if let Some(x) = maybe_reason {
                            warn!("code: {}, reason: {}", x.code, x.reason);
                        };
                        break 'ws_loop
                    },
                    m => {
                        warn!("unexpected msg received from websocket: {m:?}");
                        continue 'ws_loop
                    },
                }
            },
        }
    }

    for cid in cids {
        // NOTE: Tell the various components that a core is no longer connected.
        broadcast_channel_tx.send(BroadcastChannelMessages::Syncer(
            cid,
            SyncerBroadcastChannelMessages::CoreDisconnected,
        ))?;
        broadcast_channel_tx.send(BroadcastChannelMessages::RpcServer(
            RpcServerBroadcastChannelMessages::CoreDisconnected,
        ))?;
    }

    error!("websocket context {who} destroyed");
    Ok(()) // FIXME Too many connections error or something?
}

#[derive(Clone)]
struct AppState {
    cids: Vec<EthChainId>,
    websocket_rx: Arc<Mutex<WebSocketRx>>,
    broadcast_channel_tx: BroadcastChannelTx,
}

impl AppState {
    fn new(websocket_rx: WebSocketRx, broadcast_channel_tx: BroadcastChannelTx, cids: Vec<EthChainId>) -> Self {
        Self {
            cids,
            broadcast_channel_tx,
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
        match handle_socket(socket, addr, state.websocket_rx, state.broadcast_channel_tx, state.cids).await {
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
    config: SentinelConfig,
    broadcast_channel_tx: BroadcastChannelTx,
) -> Result<(), SentinelError> {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin/sentinel/ws_server/assets");

    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        .with_state(AppState::new(websocket_rx, broadcast_channel_tx, vec![
            config.chain_id(&BridgeSide::Native),
            config.chain_id(&BridgeSide::Host),
        ]));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // FIXME make configurable
    debug!("ws server listening on {}", addr);

    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?)
}

pub async fn ws_server_loop(
    websocket_rx: WebSocketRx,
    config: SentinelConfig,
    disable: bool,
    broadcast_channel_tx: BroadcastChannelTx,
) -> Result<(), SentinelError> {
    let name = "ws server";
    if disable {
        warn!("{name} disabled!")
    } else {
        debug!("{name} started")
    };
    let ws_server_is_enabled = !disable;

    tokio::select! {
        r = start_ws_server(
            websocket_rx,
            config.clone(),
            broadcast_channel_tx.clone(),
        ), if ws_server_is_enabled => r,
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
