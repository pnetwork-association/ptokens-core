use std::{result::Result, sync::Arc};

use common::DatabaseInterface;
use common_eth::{EthDbUtilsExt, HostDbUtils, NativeDbUtils};
use lib::{CoreAccessorMessages, CoreState, SentinelError};
use tokio::sync::{mpsc::Receiver as MpscRx, Mutex};

async fn process_message<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    msg: CoreAccessorMessages,
) -> Result<(), SentinelError> {
    let db = guarded_db.lock().await;

    match msg {
        CoreAccessorMessages::GetHostLatestBlockNumber(responder) => {
            let n = HostDbUtils::new(&*db).get_latest_eth_block_number()?;
            let _ = responder.send(Ok(n as u64));
        },
        CoreAccessorMessages::GetNativeLatestBlockNumber(responder) => {
            let n = NativeDbUtils::new(&*db).get_latest_eth_block_number()?;
            let _ = responder.send(Ok(n as u64));
        },
        CoreAccessorMessages::GetCoreState((core_type, responder)) => {
            let r = CoreState::get(&*db, &core_type)?;
            let _ = responder.send(Ok(r));
        },
        CoreAccessorMessages::GetNativeConfs(responder) => {
            let r = NativeDbUtils::new(&*db).get_eth_canon_to_tip_length_from_db()?;
            let _ = responder.send(Ok(r));
        },
        CoreAccessorMessages::GetHostConfs(responder) => {
            let r = HostDbUtils::new(&*db).get_eth_canon_to_tip_length_from_db()?;
            let _ = responder.send(Ok(r));
        },
        CoreAccessorMessages::GetLatestBlockNumbers(responder) => {
            let n = NativeDbUtils::new(&*db).get_latest_eth_block_number()?;
            let h = HostDbUtils::new(&*db).get_latest_eth_block_number()?;
            let _ = responder.send(Ok((n as u64, h as u64)));
        },
    }

    Ok(())
}

pub async fn core_accessor_loop<D: DatabaseInterface>(
    guarded_db: Arc<Mutex<D>>,
    mut core_accessor_rx: MpscRx<CoreAccessorMessages>,
) -> Result<(), SentinelError> {
    info!("core accessor listening...");

    'core_accessor_loop: loop {
        tokio::select! {
            r = core_accessor_rx.recv() => {
                if let Some(msg) = r {
                    process_message(guarded_db.clone(), msg).await?;
                    continue 'core_accessor_loop
                } else {
                    let m = "all core accessor senders dropped!";
                    warn!("{m}");
                    break 'core_accessor_loop Err(SentinelError::Custom(m.into()))
                }
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("core accessor shutting down...");
                break 'core_accessor_loop Err(SentinelError::SigInt("core accessor".into()))
            },
        }
    }
}
