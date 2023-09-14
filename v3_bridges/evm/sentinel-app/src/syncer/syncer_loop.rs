use std::result::Result;

use common_sentinel::{
    Batch,
    CoreMessages,
    EthRpcMessages,
    LatestBlockNumbers,
    SentinelConfig,
    SentinelError,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesSubmitArgs,
};
use tokio::{
    sync::mpsc::Sender as MpscTx,
    time::{sleep, Duration},
};

async fn main_loop(
    mut batch: Batch,
    config: SentinelConfig,
    _core_tx: MpscTx<CoreMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    websocket_tx: MpscTx<WebSocketMessages>,
) -> Result<(), SentinelError> {
    let side = batch.side();
    let chain_id = config.chain_id(&side);
    let validate = config.is_validating(&side);
    let pnetwork_hub = config.pnetwork_hub(&side);
    let log_prefix = format!("{} syncer", side);
    let sleep_duration = batch.get_sleep_duration();

    let latest_block_numbers = 'latest_block_getter_loop: loop {
        // NOTE: Get the core's latest block numbers for this chain
        let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::GetLatestBlockNumbers);
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
                warn!("error when getting latest block numbers in {side} syncer: {e}, retrying in {SLEEP_TIME}ms...");
                sleep(Duration::from_millis(SLEEP_TIME)).await;
                continue 'latest_block_getter_loop;
            },
        }
    };

    // NOTE: Set block number to start syncing from in the batch
    batch.set_block_num(latest_block_numbers.get_for(&chain_id)? + 1);

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
                let args = WebSocketMessagesSubmitArgs::new_for_syncer(
                    validate,
                    side,
                    chain_id.clone(),
                    pnetwork_hub,
                    batch.to_submission_material(),
                );
                let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::Submit(args));
                websocket_tx.send(msg).await?;

                let websocket_response = tokio::select! {
                    response = rx => response?,
                    _ = sleep(Duration::from_millis(*config.core().timeout())) => {
                        let m = "submitting batch for {side} {chain_id}";
                        error!("timed out whilst {m}");
                        Err(SentinelError::Timedout(m.into()))
                    }
                };
                match websocket_response {
                    Ok(WebSocketMessagesEncodable::Success(output)) => {
                        debug!("{log_prefix} websocket channel returned success output: {output}");
                        batch.increment_block_num();
                    },
                    Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoParent(e))) => {
                        let n = e.block_num;
                        warn!("{log_prefix} returned no parent err for {n}!");
                        batch.drain();
                        batch.set_block_num(n - 1);
                        batch.set_single_submissions_flag();
                        continue 'main_loop;
                    },
                    Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::BlockAlreadyInDb(e))) => {
                        let n = e.block_num;
                        warn!("{log_prefix} block {n} already in the db!");
                        batch.drain();
                        batch.set_block_num(n + 1);
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

pub async fn syncer_loop(
    batch: Batch,
    config: SentinelConfig,
    core_tx: MpscTx<CoreMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    websocket_tx: MpscTx<WebSocketMessages>,
    disable: bool,
) -> Result<(), SentinelError> {
    let side = batch.side();
    let name = format!("{side} syncer");
    if disable {
        warn!("{name} disabled!")
    } else {
        info!("starting {name}...")
    };
    let syncer_is_enabled = !disable;

    'syncer_loop: loop {
        tokio::select! {
            r = main_loop(batch.clone(), config.clone(), core_tx.clone(), eth_rpc_tx.clone(), websocket_tx.clone()), if syncer_is_enabled => {
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
                warn!("{side} syncer shutting down...");
                break 'syncer_loop Err(SentinelError::SigInt(name))
            },
            else => {
                warn!("in {name} `else` branch, {name} is currently {}abled", if syncer_is_enabled { "en" } else { "dis" });
                continue 'syncer_loop
            },
        }
    }
}
