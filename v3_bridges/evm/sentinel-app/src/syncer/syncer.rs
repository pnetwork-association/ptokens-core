use common_sentinel::{Batch, EthRpcSenders, SentinelConfig, SentinelError, SyncerBroadcastChannelMessages};

use super::{broadcast_channel_loop, syncer_loop};
use crate::type_aliases::{BroadcastChannelTx, WebSocketTx};

pub async fn syncer(
    batch: Batch,
    config: SentinelConfig,
    eth_rpc_senders: EthRpcSenders,
    websocket_tx: WebSocketTx,
    broadcast_channel_tx: BroadcastChannelTx,
    disable_syncer: bool,
) -> Result<(), SentinelError> {
    batch.check_endpoint().await?;
    let network_id = *batch.network_id();
    let eth_rpc_tx = eth_rpc_senders.sender(&network_id)?;
    let name = format!("{network_id} syncer");

    let mut core_is_connected = false;
    warn!("{name} not syncing yet due to no core connection");

    let mut syncer_is_enabled = !disable_syncer;
    if !syncer_is_enabled {
        warn!("{name} not syncer is disabled - you can enable it via an RPC call");
    };

    let core_time_limit = *config.core().timeout(); // FIXME Make configurable via RPC call

    'syncer_loop: loop {
        tokio::select! {
            r = broadcast_channel_loop(network_id, broadcast_channel_tx.subscribe()) => {
                match r {
                    Ok(msg) => {
                        let note = format!("(core is currently {}connected)", if core_is_connected { "" } else { "not "});
                        match msg {
                            SyncerBroadcastChannelMessages::Stop => {
                                debug!("msg received to stop the {name} {note}");
                                syncer_is_enabled = false;
                                continue 'syncer_loop
                            },
                            SyncerBroadcastChannelMessages::Start => {
                                debug!("msg received to start the {name} {note}");
                                syncer_is_enabled = true;
                                continue 'syncer_loop
                            },
                            SyncerBroadcastChannelMessages::CoreConnected => {
                                debug!("core connected message received in {name} {note}");
                                core_is_connected = true;
                                continue 'syncer_loop
                            },
                            SyncerBroadcastChannelMessages::CoreDisconnected => {
                                debug!("core disconnected message received in {name} {note}");
                                core_is_connected = false;
                                continue 'syncer_loop
                            },
                        }
                    },
                    Err(e) => break 'syncer_loop Err(e),
                }
            },
            r = syncer_loop(
                batch.clone(),
                config.clone(),
                eth_rpc_tx.clone(),
                websocket_tx.clone(),
                &core_is_connected,
                &core_time_limit,
            ), if core_is_connected && syncer_is_enabled => {
                match r {
                    Ok(_)  => {
                        warn!("{name} returned, restarting {name} now...");
                        continue 'syncer_loop
                    },
                    Err(SentinelError::Timedout(e)) => {
                        warn!("{name} timedout: {e}, restarting {name} now...");
                        continue 'syncer_loop
                    },
                    Err(SentinelError::NoCore) => {
                        warn!("core disconnected in {name}, restarting now...");
                        continue 'syncer_loop
                    }
                    Err(e) => {
                        warn!("{name} errored: {e}");
                        break 'syncer_loop Err(e)
                    }
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("{name} shutting down...");
                break 'syncer_loop Err(SentinelError::SigInt(name))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if syncer_is_enabled { "en" } else { "dis" });
                continue 'syncer_loop
            },
        }
    }
}
