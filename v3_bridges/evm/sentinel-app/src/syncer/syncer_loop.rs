use common_sentinel::{
    call_core,
    Batch,
    EthRpcMessages,
    LatestBlockInfos,
    SentinelConfig,
    SentinelError,
    WebSocketMessages,
    WebSocketMessagesEncodable,
    WebSocketMessagesError,
    WebSocketMessagesProcessBatchArgs,
};
use tokio::time::{sleep, Duration};

use crate::type_aliases::{EthRpcTx, WebSocketTx};

const SLEEP_TIME: u64 = 10; // FIXME make configurable

pub(super) async fn syncer_loop(
    mut batch: Batch,
    config: SentinelConfig,
    eth_rpc_tx: EthRpcTx,
    websocket_tx: WebSocketTx,
    core_is_connected: &bool,
    core_time_limit: &u64,
) -> Result<(), SentinelError> {
    let network_id = *batch.network_id();
    let log_prefix = format!("{network_id} syncer");
    let validate = matches!(config.validate(&network_id), Ok(true));
    let pnetwork_hub = config.pnetwork_hub(&network_id)?;
    let sleep_duration = batch.get_sleep_duration();

    let latest_block_numbers = 'latest_block_getter_loop: loop {
        if !core_is_connected {
            return Err(SentinelError::NoCore);
        };

        // NOTE: Get the core's latest block numbers for this chain
        let msg = WebSocketMessagesEncodable::GetLatestBlockInfos(vec![network_id]);

        let r = match call_core(*core_time_limit, websocket_tx.clone(), msg).await {
            Ok(WebSocketMessagesEncodable::Error(WebSocketMessagesError::NotInitialized(nid))) => {
                warn!("{nid} not intialized, you can initialize it via RPC call - rechecking in {SLEEP_TIME}s...");
                sleep(Duration::from_secs(SLEEP_TIME)).await;
                continue 'latest_block_getter_loop;
            },
            Ok(x) => x,
            Err(e) => {
                error!("{e}");
                return Err(e);
            },
        };

        break 'latest_block_getter_loop LatestBlockInfos::try_from(r)?;
    };

    // NOTE: Set block number to start syncing from in the batch
    batch.set_block_num(latest_block_numbers.get_for(&network_id)?.block_number() + 1);

    'main_loop: loop {
        if !core_is_connected {
            return Err(SentinelError::NoCore);
        };

        let (msg, rx) = EthRpcMessages::get_sub_mat_msg(network_id, batch.get_block_num());
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
                    network_id,
                    pnetwork_hub,
                    batch.to_submission_material(),
                    *batch.governance_address(),
                );
                let (msg, rx) = WebSocketMessages::new(WebSocketMessagesEncodable::ProcessBatch(args));
                websocket_tx.send(msg).await?;

                let websocket_response = tokio::select! {
                    response = rx => response?,
                    _ = sleep(Duration::from_secs(*core_time_limit)) => {
                        let m = "submitting batch for {side} {network_id}";
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
                sleep(Duration::from_secs(SLEEP_TIME)).await;
                continue 'main_loop;
            },
            Err(e) => break 'main_loop Err(e),
        }
    }
}
