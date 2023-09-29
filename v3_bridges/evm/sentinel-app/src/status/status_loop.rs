use common_sentinel::{
    BroadcastChannelMessages,
    SentinelConfig,
    SentinelError,
    StatusBroadcastChannelMessages,
    StatusMessages,
    WebSocketMessages,
};
use tokio::{
    sync::{
        broadcast::{Receiver as MpMcRx, Sender as MpMcTx},
        mpsc::{Receiver as MpscRx, Sender as MpscTx},
    },
    time::{sleep, Duration},
};

async fn broadcast_channel_loop(
    mut broadcast_channel_rx: MpMcRx<BroadcastChannelMessages>,
) -> Result<StatusBroadcastChannelMessages, SentinelError> {
    // NOTE: This loops continuously listening to the broadcasting channel, and only returns if we
    // receive a pertinent message. This way, other messages won't cause early returns in the main
    // tokios::select, so then the main_loop can continue doing its work.
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::Status(msg)) => break 'broadcast_channel_loop Ok(msg),
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for us
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

async fn publish_status_loop(frequency: u64, status_tx: MpscTx<StatusMessages>) -> Result<(), SentinelError> {
    // NOTE: This loop runs to send messages to the status loop at a configruable frequency to tell
    // it to publish its status. It should never return, except in error.
    'publish_status_loop: loop {
        sleep(Duration::from_secs(frequency)).await;
        warn!("{frequency}s has elapsed - sending message to cancel any cancellable user ops");
        /*
        match broadcaster_tx.send(BroadcasterMessages::CancelUserOps).await {
            Ok(_) => continue 'publish_status_loop,
            Err(e) => break 'publish_status_loop Err(e.into()),
        }
        */
    }
}

const PUBLISHING_FREQUENCY: u64 = 120; // FIXME Make configurable

pub async fn status_loop(
    config: SentinelConfig,
    mut status_rx: MpscRx<StatusMessages>,
    status_tx: MpscTx<StatusMessages>,
    broadcast_channel_tx: MpMcTx<BroadcastChannelMessages>,
    websocket_tx: MpscTx<WebSocketMessages>,
) -> Result<(), SentinelError> {
    let name = "status loop";

    let mut core_is_connected = false;
    let mut status_is_enabled = false;

    todo!("impl an async function to publish the status, and add it to the below");

    'status_loop: loop {
        tokio::select! {
            r = publish_status_loop(PUBLISHING_FREQUENCY, status_tx.clone()) => {
                let sleep_time = 30; // FIXME make configurable
                match r {
                    Ok(_) => {
                        warn!("publish status loop returned Ok(()) for some reason");
                    },
                    Err(e) => {
                        error!("publish status publisher loop error: {e}");
                    }
                }
                warn!("sleeping for {sleep_time}s and restarting status loop");
                sleep(Duration::from_secs(sleep_time)).await;
                continue 'status_loop
            },
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe()) => {
                match r {
                    Err(e) => break 'status_loop Err(e),
                    Ok(msg) => {
                        let note = format!("(core is currently {}connected)", if core_is_connected { "" } else { "not "});
                        match msg {
                            StatusBroadcastChannelMessages::Stop => {
                                warn!("msg received to stop the {name} {note}");
                                status_is_enabled = false;
                                continue 'status_loop
                            },
                            StatusBroadcastChannelMessages::Start => {
                                warn!("msg received to start the {name} {note}");
                                status_is_enabled = true;
                                continue 'status_loop
                            },
                            StatusBroadcastChannelMessages::CoreConnected => {
                                warn!("core connected message received in {name} {note}");
                                core_is_connected = true;
                                continue 'status_loop
                            },
                            StatusBroadcastChannelMessages::CoreDisconnected => {
                                warn!("core disconnected message received in {name} {note}");
                                core_is_connected = false;
                                continue 'status_loop
                            },
                        }
                    },
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'status_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if status_is_enabled { "en" } else { "dis" });
                continue 'status_loop
            },
        }
    }

    /*

    warn!("{name} not active yet due to no core connection");

    Env::init()?;
    let host_broadcaster_pk = Env::get_host_broadcaster_private_key()?;
    let native_broadcaster_pk = Env::get_native_broadcaster_private_key()?;

    'broadcaster_loop: loop {
        tokio::select! {
            r = cancellation_loop(CANCELLABLE_OPS_CHECK_FREQUENCY, broadcaster_tx.clone()), if (broadcaster_is_enabled && core_is_connected) => {
                let sleep_time = 30;
                match r {
                    Ok(_) => {
                        warn!("broadcaster cancellation loop returned Ok(()) for some reason");
                    },
                    Err(e) => {
                        error!("broadcaster cancellation loop error: {e}");
                    }
                }
                warn!("sleeping for {sleep_time}s and restarting broadcaster loop");
                sleep(Duration::from_secs(sleep_time)).await;
                continue 'broadcaster_loop
            },
            r = broadcast_channel_loop(broadcast_channel_tx.subscribe()) => {
                match r {
                    Ok(msg) => {
                        let note = format!("(core is currently {}connected)", if core_is_connected { "" } else { "not "});
                        match msg {
                            BroadcasterBroadcastChannelMessages::Stop => {
                                warn!("msg received to stop the {name} {note}");
                                broadcaster_is_enabled = false;
                                continue 'broadcaster_loop
                            },
                            BroadcasterBroadcastChannelMessages::Start => {
                                warn!("msg received to start the {name} {note}");
                                broadcaster_is_enabled = true;
                                continue 'broadcaster_loop
                            },
                            BroadcasterBroadcastChannelMessages::CoreConnected => {
                                warn!("core connected message received in {name} {note}");
                                core_is_connected = true;
                                continue 'broadcaster_loop
                            },
                            BroadcasterBroadcastChannelMessages::CoreDisconnected => {
                                warn!("core disconnected message received in {name} {note}");
                                core_is_connected = false;
                                continue 'broadcaster_loop
                            },
                        }
                    },
                    Err(e) => break 'broadcaster_loop Err(e),
                }
            },
            r = rx.recv() , if broadcaster_is_enabled && core_is_connected => match r {
                Some(BroadcasterMessages::CancelUserOps) => {
                    match cancel_user_ops(
                        &config,
                        websocket_tx.clone(),
                        eth_rpc_tx.clone(),
                        &native_broadcaster_pk,
                        &host_broadcaster_pk,
                    ).await {
                        Ok(_) => {
                            info!("finished handling user op cancellation request");
                        }
                        Err(SentinelError::UserOp(boxed_user_op_error)) => match *boxed_user_op_error {
                            UserOpError::InsufficientBalance { have, need } => {
                                error!("!!! WARNING !!!");
                                error!("!!! WARNING !!!");
                                error!("!!! WARNING !!!");
                                warn!(">>> insufficient balance to cancel a user op - have: {have}, need: {need} <<<");
                                error!("!!! WARNING !!!");
                                error!("!!! WARNING !!!");
                                error!("!!! WARNING !!!");
                                continue 'broadcaster_loop
                            },
                            e => {
                                error!("unhandled user op error: {e}");
                                break 'broadcaster_loop Err(e.into())
                            }
                        },
                        Err(e) => {
                            error!("{e}");
                        }
                    };
                    continue 'broadcaster_loop
                },
                None => {
                    let m = "all {name} senders dropped!";
                    warn!("{m}");
                    break 'broadcaster_loop Err(SentinelError::Custom(name.into()))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'broadcaster_loop Err(SentinelError::SigInt(name.into()))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if broadcaster_is_enabled { "en" } else { "dis" });
                continue 'broadcaster_loop
            },
        }
    }
    */
}
