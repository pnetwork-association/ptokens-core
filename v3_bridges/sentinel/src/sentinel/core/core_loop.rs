use std::{result::Result, sync::Arc};

use common::{BridgeSide, DatabaseInterface};
use common_eth::{EthDbUtilsExt, HostDbUtils, NativeDbUtils};
use lib::{CoreMessages, CoreState, SentinelDbUtils, SentinelError, UserOpList, USER_OP_CANCEL_TX_GAS_LIMIT};
use tokio::sync::{mpsc::Receiver as MpscRx, Mutex};

async fn process_message<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    msg: CoreMessages,
) -> Result<(), SentinelError> {
    let db = guarded_db.lock().await;

    match msg {
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
        CoreMessages::GetAddress { side, responder } => {
            let a = match side {
                BridgeSide::Native => HostDbUtils::new(&*db).get_public_eth_address_from_db()?,
                BridgeSide::Host => NativeDbUtils::new(&*db).get_public_eth_address_from_db()?,
            };
            let _ = responder.send(Ok(a));
        },
        CoreMessages::GetCancellationTx {
            op,
            gas_price,
            nonce,
            responder,
            state_manager,
        } => {
            let h = HostDbUtils::new(&*db);
            let n = HostDbUtils::new(&*db);
            let (chain_id, pk) = match op.side() {
                BridgeSide::Host => (h.get_eth_chain_id_from_db()?, h.get_eth_private_key_from_db()?),
                BridgeSide::Native => (n.get_eth_chain_id_from_db()?, n.get_eth_private_key_from_db()?),
            };
            let tx = op.cancel(
                nonce,
                gas_price,
                &state_manager,
                USER_OP_CANCEL_TX_GAS_LIMIT as usize,
                &pk,
                &chain_id,
            )?;
            let _ = responder.send(Ok(tx));
        },
    }

    Ok(())
}

pub async fn core_loop<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    mut core_rx: MpscRx<CoreMessages>,
) -> Result<(), SentinelError> {
    info!("core listening...");

    'core_loop: loop {
        tokio::select! {
            r = core_rx.recv() => {
                if let Some(msg) = r {
                    process_message(guarded_db.clone(), msg).await?;
                    continue 'core_loop
                } else {
                    let m = "all core senders dropped!";
                    warn!("{m}");
                    break 'core_loop Err(SentinelError::Custom(m.into()))
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("core shutting down...");
                break 'core_loop Err(SentinelError::SigInt("core".into()))
            },
        }
    }
}