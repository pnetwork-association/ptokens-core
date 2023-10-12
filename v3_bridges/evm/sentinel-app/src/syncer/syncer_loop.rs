use std::result::Result;

use common_metadata::MetadataChainId;
use common_sentinel::{
    Batch,
    BroadcastChannelMessages,
    EthRpcMessages,
    LatestBlockNumbers,
    SentinelConfig,
    SentinelError,
    SyncerBroadcastChannelMessages,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesProcessBatchArgs,
};
use tokio::{
    sync::{
        broadcast::{Receiver as MpMcRx, Sender as MpMcTx},
        mpsc::Sender as MpscTx,
    },
    time::{sleep, Duration},
};

async fn main_loop(
    mut batch: Batch,
    config: SentinelConfig,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    websocket_tx: MpscTx<WebSocketMessages>,
) -> Result<(), SentinelError> {
    let side = *batch.side();
    let mcid = *batch.mcid();
    let log_prefix = format!("{mcid} syncer");
    let validate = config.is_validating(&side);
    let pnetwork_hub = config.pnetwork_hub(&side);
    let sleep_duration = batch.get_sleep_duration();

    let latest_block_numbers = 'latest_block_getter_loop: loop {
        // NOTE: Get the core's latest block numbers for this chain
        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetLatestBlockNumbers(vec![*batch.mcid()]));
        websocket_tx.send(msg).await?;

        let websocket_response = tokio::select! {
            response = rx => response?,
            _ = sleep(Duration::from_millis(*config.core().timeout())) => {
                let m = "getting latest block numbers in {side} syncer";
                error!("timed out whilst {m}");
                Err(SentinelError::Timedout(m.into()))
            }
        }?;
        match LatestBlockNumbers::try_from(websocket_response) {
            Ok(x) => break 'latest_block_getter_loop x,
            Err(e) => {
                const SLEEP_TIME: u64 = 10_000; // FIXME make configurable
                warn!("error when getting latest block numbers in {log_prefix}: {e}, retrying in {SLEEP_TIME}ms...");
                sleep(Duration::from_millis(SLEEP_TIME)).await;
                continue 'latest_block_getter_loop;
            },
        }
    };

    // NOTE: Set block number to start syncing from in the batch
    batch.set_block_num(latest_block_numbers.get_for(batch.mcid())? + 1);

    'main_loop: loop {
        let (msg, rx) = EthRpcMessages::get_sub_mat_msg(side, batch.get_block_num());
        eth_rpc_tx.send(msg).await?;
        match rx.await? {
            Ok(block) => {
                batch.push(block);
                if !batch.is_ready_to_submit() {
                    batch.increment_block_num();
                    continue 'main_loop;
                }
                // TODO check if batch is chained correctly!
                info!("{log_prefix} batch is ready to submit!");
                let args = WebSocketMessagesProcessBatchArgs::new_for_syncer(
                    validate,
                    side,
                    mcid,
                    pnetwork_hub,
                    batch.to_submission_material(),
                    *batch.governance_address(),
                );
                let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::ProcessBatch(args));
                websocket_tx.send(msg).await?;

                let websocket_response = tokio::select! {
                    response = rx => response?,
                    _ = sleep(Duration::from_millis(*config.core().timeout())) => {
                        let m = "submitting batch for {side} {mcid}";
                        error!("timed out whilst {m}");
                        Err(SentinelError::Timedout(m.into()))
                    }
                };
                match websocket_response {
                    Ok(WebSocketMessagesEncodable::Success(output)) => {
                        debug!("{log_prefix} websocket channel returned success output: {output}");
                        batch.update_bpm_from_json(output);
                        batch.increment_block_num();
                    },
                    Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoParent(e))) => {
                        let n = e.block_num();
                        warn!("{log_prefix} returned no parent err for {n}!");
                        batch.drain();
                        batch.set_block_num(n - 1);
                        batch.set_single_submissions_flag();
                        continue 'main_loop;
                    },
                    Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::BlockAlreadyInDb { num, .. })) => {
                        warn!("{log_prefix} block {num} already in the db!");
                        batch.drain();
                        batch.set_block_num(num + 1);
                        batch.set_single_submissions_flag();
                        continue 'main_loop;
                    },
                    Ok(r) => {
                        let msg = format!("{log_prefix} received unexpected websocket response {r}");
                        error!("{msg}");
                        break 'main_loop Err(WebSocketMessagesError::UnexpectedResponse(msg).into());
                    },
                    Err(e) => {
                        warn!("{log_prefix} oneshot channel returned err {e}");
                        break 'main_loop Err(e);
                    },
                };

                batch.drain();
                continue 'main_loop;
            },
            Err(SentinelError::NoBlock(_)) => {
                info!("{log_prefix} no next block yet - sleeping for {sleep_duration}ms...");
                sleep(Duration::from_millis(sleep_duration)).await;
                continue 'main_loop;
            },
            Err(e) => break 'main_loop Err(e),
        }
    }
}

async fn broadcast_channel_loop(
    mcid: MetadataChainId,
    mut broadcast_channel_rx: MpMcRx<BroadcastChannelMessages>,
) -> Result<SyncerBroadcastChannelMessages, SentinelError> {
    // NOTE: This loops continuously listening to the broadcasting channel, and only returns if we
    // receive a pertinent message. This way, other messages won't cause early returns in the main
    // tokios::select, so then the main_loop can continue doing its work.
    'broadcast_channel_loop: loop {
        match broadcast_channel_rx.recv().await {
            Ok(BroadcastChannelMessages::Syncer(msg_mcid, msg)) => {
                // NOTE: We have a syncer message...
                if mcid == msg_mcid {
                    // ...and it's for this syncer so we return it
                    break 'broadcast_channel_loop Ok(msg);
                } else {
                    // ...but it's not for this syncer so we go back to listening on the receiver
                    debug!("syncer message: '{msg}' for mcid: '{mcid}' ignored");
                    continue 'broadcast_channel_loop;
                }
            },
            Ok(_) => continue 'broadcast_channel_loop, // NOTE: The message wasn't for the syncer
            Err(e) => break 'broadcast_channel_loop Err(e.into()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn syncer_loop(
    batch: Batch,
    config: SentinelConfig,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    websocket_tx: MpscTx<WebSocketMessages>,
    broadcast_channel_tx: MpMcTx<BroadcastChannelMessages>,
) -> Result<(), SentinelError> {
    batch.check_endpoint().await?;

    let mcid = *batch.mcid();
    let name = format!("{mcid} syncer");

    let mut core_is_connected = false;
    let mut syncer_is_enabled = false;

    warn!("{name} not syncing yet due to no core connection and being disabled");

    'syncer_loop: loop {
        tokio::select! {
            r = broadcast_channel_loop(mcid, broadcast_channel_tx.subscribe()) => {
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
            r = main_loop(
                batch.clone(),
                config.clone(),
                eth_rpc_tx.clone(),
                websocket_tx.clone(),
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
