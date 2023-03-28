use std::{result::Result, sync::Arc};

use common::DatabaseInterface;
use common_eth::{EthDbUtilsExt, HostDbUtils, NativeDbUtils};
use lib::{CoreMessages, CoreState, SentinelDbUtils, SentinelError, UnmatchedUserOps};
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
        CoreMessages::GetUnmatchedUserOps(responder) => {
            let db_utils = SentinelDbUtils::new(&*db);
            let n = db_utils.get_native_user_operations()?;
            let h = db_utils.get_native_user_operations()?;
            let _ = responder.send(Ok(UnmatchedUserOps::new(n, h)));
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
