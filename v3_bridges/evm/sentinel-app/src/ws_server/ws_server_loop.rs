use std::{net::SocketAddr, ops::ControlFlow, path::PathBuf, result::Result};

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
use futures::stream::StreamExt;
use common_sentinel::{CoreMessages, SentinelConfig, SentinelError};
use tokio::sync::mpsc::Sender as MpscTx;
use tower_http::services::ServeDir;

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    // FIXME Return result
    if socket.send(Message::Ping(vec![1, 3, 3, 7])).await.is_ok() {
        debug!("pinged {}...", who);
    } else {
        error!("could not send ping {}!", who);
        // NOTE: No Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // NOTE: Receive single message from a client (we can either receive or send with socket).
    // this will likely be the Pong for our Ping or a hello message from client.
    // waiting for message from a client will block this task, but will not block other client's
    // connections.
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, who).is_break() {
                return;
            }
        } else {
            error!("client {who} abruptly disconnected");
            return;
        }
    }

    // Since each client gets individual statemachine, we can pause handling
    // when necessary to wait for some external event (in this case illustrated by sleeping).
    // Waiting for this client to finish getting its greetings does not prevent other clients from
    // connecting to server and receiving their greetings.
    /*
    for i in 1..5 {
        if socket
            .send(Message::Text(format!("Hi {i} times!")))
            .await
            .is_err()
        {
            debug!("client {who} abruptly disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    */

    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();

    // Spawn a task that will push several messages to the client (does not matter what client does)
    /*
    let mut send_task = tokio::spawn(async move {
        let n_msg = 20;
        for i in 0..n_msg {
            // In case of any websocket error, we exit.
            if sender
                .send(Message::Text(format!("Server message {i} ...")))
                .await
                .is_err()
            {
                return i;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        debug!("Sending close to {who}...");
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: Cow::from("Goodbye"),
            })))
            .await
        {
            debug!("Could not send Close due to {}, probably it is ok?", e);
        }
        n_msg
    });
    */

    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            if process_message(msg, who).is_break() {
                break;
            }
        }
        cnt
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        /*
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => debug!("{} messages sent to {}", a, who),
                Err(a) => debug!("Error sending messages {:?}", a)
            }
            recv_task.abort();
        },
        */
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(n) => debug!("received {n} messages"),
                Err(e) => debug!("error receiving messages {e}")
            }
            //send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    error!("websocket context {} destroyed", who);
}

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

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(core_tx): State<MpscTx<CoreMessages>>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    debug!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn start_ws_server(core_tx: MpscTx<CoreMessages>, config: SentinelConfig) -> Result<(), SentinelError> {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin/sentinel/ws_server/assets");
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        .with_state(core_tx);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // FIXME make configurable
    debug!("ws server listening on {}", addr);

    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?)
}

pub async fn ws_server_loop(
    core_tx: MpscTx<CoreMessages>,
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

    'ws_server_loop: loop {
        tokio::select! {
            r = start_ws_server(core_tx.clone(), config.clone()), if ws_server_is_enabled => {
                if r.is_ok() {
                    warn!("{name} returned, restarting {name} now...");
                    continue 'ws_server_loop
                } else {
                    break 'ws_server_loop r
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'ws_server_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if ws_server_is_enabled { "en" } else { "dis" });
                continue 'ws_server_loop
            }
        }
    }
}
