use std::{net::SocketAddr, path::PathBuf, result::Result, sync::Arc, time::Duration};

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
use common_sentinel::{
    BroadcastChannelMessages,
    ChallengeResponderBroadcastChannelMessages,
    NetworkId,
    RpcServerBroadcastChannelMessages,
    SentinelConfig,
    SentinelError,
    StatusPublisherBroadcastChannelMessages,
    SyncerBroadcastChannelMessages,
    UserOpCancellerBroadcastChannelMessages,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
};
use derive_getters::Getters;
use futures::{stream::StreamExt, SinkExt};
use tokio::{sync::Mutex, time::sleep};
use tower_http::services::ServeDir;

use crate::type_aliases::{BroadcastChannelTx, WebSocketRx};

async fn handle_socket(
    mut socket: WebSocket,
    who: SocketAddr,
    websocket_rx: Arc<Mutex<WebSocketRx>>,
    broadcast_channel_tx: BroadcastChannelTx,
    network_ids: Vec<NetworkId>,
) -> Result<(), SentinelError> {
    if socket.send(Message::Ping(vec![1, 3, 3, 7])).await.is_ok() {
        debug!("pinged {}...", who);
    } else {
        error!("could not send ping {}!", who);
        // NOTE: No Error here since the only thing we can do is to close the connection.
        // If we cannot send messages, there is no way to salvage the statemachine anyway.
        return Ok(());
    }

    // NOTE: Await a response to our ping...
    match socket.recv().await {
        Some(Ok(Message::Pong(v))) => debug!("{} sent pong with {:?}", who, v),
        Some(Ok(Message::Close(maybe_close_frame))) => {
            if let Some(cf) = maybe_close_frame {
                debug!("{} sent close with code {} and reason `{}`", who, cf.code, cf.reason);
            } else {
                debug!("{} somehow sent close message without CloseFrame", who);
            };
            return Ok(());
        },
        _ => {
            error!("did not receive expected response to ping - terminating connection");
            return Ok(());
        },
    };

    let (mut sender, mut receiver) = socket.split();

    // NOTE: Trying the lock here limits us to one active connection.
    let mut rx = websocket_rx.try_lock()?;

    // FIXME Need a better way/single location for all the services that need to know about this
    // core cxn status
    for network_id in network_ids.iter().cloned() {
        // NOTE: Tell the various components that a core is connected.
        broadcast_channel_tx.send(BroadcastChannelMessages::Syncer(
            network_id,
            SyncerBroadcastChannelMessages::CoreConnected,
        ))?;
    }
    broadcast_channel_tx.send(BroadcastChannelMessages::RpcServer(
        RpcServerBroadcastChannelMessages::CoreConnected,
    ))?;
    broadcast_channel_tx.send(BroadcastChannelMessages::UserOpCanceller(
        UserOpCancellerBroadcastChannelMessages::CoreConnected,
    ))?;
    broadcast_channel_tx.send(BroadcastChannelMessages::StatusPublisher(
        StatusPublisherBroadcastChannelMessages::CoreConnected,
    ))?;
    broadcast_channel_tx.send(BroadcastChannelMessages::ChallengeResponder(
        ChallengeResponderBroadcastChannelMessages::CoreConnected,
    ))?;

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
                        if m.is_none() {
                            // NOTE: If the core unexpected disconnects with no message, we enter
                            // an infinite loop here, with this msg being `None`. As such, lets
                            // break out instead and kill the connection.
                            break 'ws_loop
                        } else {
                            warn!("unexpected msg received from websocket: {m:?}");
                            continue 'ws_loop
                        }
                    },
                }
            },
        }
    }

    for network_id in network_ids {
        // NOTE: Tell the various components that a core is no longer connected.
        broadcast_channel_tx.send(BroadcastChannelMessages::Syncer(
            network_id,
            SyncerBroadcastChannelMessages::CoreDisconnected,
        ))?;
    }
    broadcast_channel_tx.send(BroadcastChannelMessages::RpcServer(
        RpcServerBroadcastChannelMessages::CoreDisconnected,
    ))?;
    broadcast_channel_tx.send(BroadcastChannelMessages::UserOpCanceller(
        UserOpCancellerBroadcastChannelMessages::CoreDisconnected,
    ))?;
    broadcast_channel_tx.send(BroadcastChannelMessages::StatusPublisher(
        StatusPublisherBroadcastChannelMessages::CoreDisconnected,
    ))?;
    broadcast_channel_tx.send(BroadcastChannelMessages::ChallengeResponder(
        ChallengeResponderBroadcastChannelMessages::CoreDisconnected,
    ))?;

    error!("websocket context {who} destroyed");
    Ok(())
}

#[derive(Clone, Getters)]
struct AppState {
    network_ids: Vec<NetworkId>,
    websocket_rx: Arc<Mutex<WebSocketRx>>,
    broadcast_channel_tx: BroadcastChannelTx,
}

impl AppState {
    fn new(websocket_rx: WebSocketRx, broadcast_channel_tx: BroadcastChannelTx, network_ids: Vec<NetworkId>) -> Self {
        Self {
            network_ids,
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
        match handle_socket(
            socket,
            addr,
            state.websocket_rx,
            state.broadcast_channel_tx,
            state.network_ids,
        )
        .await
        {
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
        .with_state(AppState::new(websocket_rx, broadcast_channel_tx, config.network_ids()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // FIXME make configurable
    debug!("ws server listening on {}", addr);

    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?)
}

pub async fn ws_server_loop(
    websocket_rx: WebSocketRx,
    config: SentinelConfig,
    broadcast_channel_tx: BroadcastChannelTx,
) -> Result<(), SentinelError> {
    let name = "ws server";
    let ws_server_is_enabled = false;

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
