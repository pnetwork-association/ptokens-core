use std::{result::Result, sync::Arc};

use common::BridgeSide;
use common_eth::{EthDbUtilsExt, HostDbUtils, NativeDbUtils};
use common_sentinel::{
    process_batch,
    BroadcasterMessages,
    Bytes4,
    CoreMessages,
    CoreState,
    Heartbeats,
    MongoMessages,
    NetworkId,
    SentinelConfig,
    SentinelDbUtils,
    SentinelError,
    UserOpList,
    HOST_PROTOCOL_ID,
    NATIVE_PROTOCOL_ID,
};
use serde_json::json;
use tokio::sync::{
    mpsc::{Receiver as MpscRx, Sender as MpscTx},
    Mutex,
};

lazy_static! {
    // NOTE: This is just used to give a quick RPC-access way to see how fast the sentinel is syncing
    static ref HEARTBEATS: std::sync::Mutex<Heartbeats> = std::sync::Mutex::new(Heartbeats::new());
}

/*
async fn handle_message(
    config: &SentinelConfig,
    msg: CoreMessages,
    mongo_tx: MpscTx<MongoMessages>,
    broadcaster_tx: MpscTx<BroadcasterMessages>,
    n_origin_network_id: Bytes4,
    h_origin_network_id: Bytes4,
) -> Result<(), SentinelError> {
    let reprocess = false;
    let db = guarded_db.lock().await;

    match msg {
        CoreMessages::Process(args) => {
            let side = args.side();
            debug!("processing {side} material...");
            // NOTE If we match on the process fxn call directly, we get tokio errors!
            let result = process_batch(
                &*db,
                &config.pnetwork_hub(&side),
                &args.batch,
                config.is_validating(&side),
                side,
                if side.is_native() {
                    &n_origin_network_id
                } else {
                    &h_origin_network_id
                },
                reprocess,
            );
            match result {
                Ok(output) => {
                    let _ = args.responder.send(Ok(())); // NOTE: Send an OK response so syncer can continue

                    let maybe_json = match HEARTBEATS.lock() {
                        Ok(mut h) => {
                            h.push(&output);
                            Some(h.to_json())
                        },
                        Err(e) => {
                            // NOTE: If for some reason the lock gets poisoned, we don't care too
                            // much since the heartbeats is just a crude monitoring method.
                            error!("cannot push latest info into heartbeats: {e}");
                            None
                        },
                    };
                    if let Some(json) = maybe_json {
                        mongo_tx.send(MongoMessages::PutHeartbeats(json)).await?
                    }

                    if output.has_user_ops() {
                        // NOTE: Some user ops were processed so there may be some that
                        // are cancellable.
                        broadcaster_tx.send(BroadcasterMessages::CancelUserOps).await?;
                    }

                    return Ok(());
                },
                Err(SentinelError::NoParent(e)) => {
                    debug!("{side} no parent error successfully caught and returned to syncer");
                    let _ = args.responder.send(Err(SentinelError::NoParent(e)));
                    return Ok(());
                },
                Err(SentinelError::BlockAlreadyInDb(e)) => {
                    debug!("{side} block already in db successfully caught and returned to syncer");
                    let _ = args.responder.send(Err(SentinelError::BlockAlreadyInDb(e)));
                    return Ok(());
                },
                Err(e) => {
                    warn!("{side} processor err: {e}");
                    return Err(e);
                },
            }
        },
        CoreMessages::GetCancellableUserOps(responder) => {
            let sentinel_db_utils = SentinelDbUtils::new(&*db);
            let h_latest_timestamp = HostDbUtils::new(&*db).get_latest_eth_block_timestamp()?;
            let n_latest_timestamp = NativeDbUtils::new(&*db).get_latest_eth_block_timestamp()?;
            let r = UserOpList::get(&sentinel_db_utils).get_cancellable_ops(
                config.core().max_cancellable_time_delta(),
                &sentinel_db_utils,
                n_latest_timestamp,
                h_latest_timestamp,
            );
            let _ = responder.send(r);
        },
        CoreMessages::RemoveUserOp { uid, responder } => {
            let db_utils = SentinelDbUtils::new(&*db);
            let mut list = UserOpList::get(&db_utils);
            let removed_from_list = list.remove_entry(&db_utils, &uid)?;
            let r = json!({ "uid": uid, "removed_from_list": removed_from_list });
            let _ = responder.send(Ok(r));
        },
        CoreMessages::GetHostLatestBlockNumber(responder) => {
            let n = HostDbUtils::new(&*db).get_latest_eth_block_number()?;
            let _ = responder.send(Ok(n as u64));
        },
        CoreMessages::GetNativeLatestBlockNumber(responder) => {
            let n = NativeDbUtils::new(&*db).get_latest_eth_block_number()?;
            let _ = responder.send(Ok(n as u64));
        },
        CoreMessages::GetCoreState((core_type, responder)) => {
            let r = CoreState::get(&*db, &core_type)?;
            let _ = responder.send(Ok(r));
        },
        CoreMessages::GetNativeConfs(responder) => {
            let r = NativeDbUtils::new(&*db).get_eth_canon_to_tip_length_from_db()?;
            let _ = responder.send(Ok(r));
        },
        CoreMessages::GetHostConfs(responder) => {
            let r = HostDbUtils::new(&*db).get_eth_canon_to_tip_length_from_db()?;
            let _ = responder.send(Ok(r));
        },
        CoreMessages::GetLatestBlockNumbers(responder) => {
            let n = NativeDbUtils::new(&*db).get_latest_eth_block_number()?;
            let h = HostDbUtils::new(&*db).get_latest_eth_block_number()?;
            let _ = responder.send(Ok((n as u64, h as u64)));
        },
        CoreMessages::GetUserOps(responder) => {
            let ops = UserOpList::user_ops(&SentinelDbUtils::new(&*db))?;
            let _ = responder.send(Ok(ops));
        },
        CoreMessages::GetUserOpList(responder) => {
            let l = UserOpList::get(&SentinelDbUtils::new(&*db));
            let _ = responder.send(Ok(l));
        },
        CoreMessages::GetGasPrices(responder) => {
            let n = NativeDbUtils::new(&*db).get_eth_gas_price_from_db()?;
            let h = HostDbUtils::new(&*db).get_eth_gas_price_from_db()?;
            let _ = responder.send(Ok((n, h)));
        },
        CoreMessages::GetAddress { side, responder } => {
            let a = match side {
                BridgeSide::Native => NativeDbUtils::new(&*db).get_public_eth_address_from_db()?,
                BridgeSide::Host => HostDbUtils::new(&*db).get_public_eth_address_from_db()?,
            };
            let _ = responder.send(Ok(a));
        },
        CoreMessages::GetCancellationTx {
            op,
            nonce,
            gas_price,
            gas_limit,
            responder,
            pnetwork_hub,
            broadcaster_pk,
        } => {
            let h = HostDbUtils::new(&*db);
            let n = NativeDbUtils::new(&*db);
            let side = op.destination_side();
            let (chain_id, core_pk) = if side.is_native() {
                (n.get_eth_chain_id_from_db()?, n.get_eth_private_key_from_db()?)
            } else {
                (h.get_eth_chain_id_from_db()?, h.get_eth_private_key_from_db()?)
            };
            debug!("core cancellation getter chain ID: {chain_id}");
            let tx = op.get_cancellation_tx(
                nonce,
                gas_price,
                gas_limit,
                &pnetwork_hub,
                &chain_id,
                &core_pk,
                &broadcaster_pk,
            )?;
            let _ = responder.send(Ok(tx));
        },
    }

    Ok(())
}
*/

pub async fn core_loop(
    config: SentinelConfig,
    mut core_rx: MpscRx<CoreMessages>,
    mongo_tx: MpscTx<MongoMessages>,
    broadcaster_tx: MpscTx<BroadcasterMessages>,
) -> Result<(), SentinelError> {
    info!("core listening...");
    let h_origin_network_id = NetworkId::new(config.host().get_eth_chain_id(), *HOST_PROTOCOL_ID).to_bytes_4()?;
    let n_origin_network_id = NetworkId::new(config.native().get_eth_chain_id(), *NATIVE_PROTOCOL_ID).to_bytes_4()?;

    'core_loop: loop {
        tokio::select! {
            /*
            r = core_rx.recv() => {
                if let Some(msg) = r {
                    handle_message(
                        &config,
                        msg,
                        mongo_tx.clone(),
                        broadcaster_tx.clone(),
                        n_origin_network_id,
                        h_origin_network_id,
                    ).await?;
                    continue 'core_loop
                } else {
                    let m = "all core senders dropped!";
                    warn!("{m}");
                    break 'core_loop Err(SentinelError::Custom(m.into()))
                }
            },
            */
            _ = tokio::signal::ctrl_c() => {
                warn!("core shutting down...");
                break 'core_loop Err(SentinelError::SigInt("core".into()))
            },
        }
    }
}
