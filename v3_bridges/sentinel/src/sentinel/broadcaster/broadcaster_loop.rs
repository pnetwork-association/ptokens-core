use std::result::Result;

use lib::{BroadcasterMessages, SentinelConfig, SentinelError};
use tokio::sync::mpsc::Receiver as MpscRx;

pub async fn broadcaster_loop(
    mut rx: MpscRx<BroadcasterMessages>,
    config: SentinelConfig,
) -> Result<(), SentinelError> {
    let _host_endpoints = config.get_host_endpoints();
    let _native_endpoints = config.get_native_endpoints();

    'broadcaster_loop: loop {
        tokio::select! {
            r = rx.recv() => match r {
                _ => {
                    let m = "all broadcaster senders dropped!";
                    warn!("{m}");
                    break 'broadcaster_loop Err(SentinelError::Custom(m.into()))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("broadcaster shutting down...");
                break 'broadcaster_loop Err(SentinelError::SigInt("broadcaster".into()))
            },
        }
    }
}
