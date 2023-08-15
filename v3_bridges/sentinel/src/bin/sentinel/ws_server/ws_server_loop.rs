use std::result::Result;

use lib::{CoreMessages, SentinelConfig, SentinelError};
use tokio::sync::mpsc::Sender as MpscTx;

async fn start_ws_server(core_tx: MpscTx<CoreMessages>, config: SentinelConfig) -> Result<(), SentinelError> {
    Ok(())
}

pub async fn ws_server_loop(
    core_tx: MpscTx<CoreMessages>,
    config: SentinelConfig,
    disable: bool,
) -> Result<(), SentinelError> {
    let name = "ws server";
    if disable {
        warn!("{name} disabled!")
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
