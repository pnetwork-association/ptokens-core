use std::result::Result;

use lib::{EthRpcMessages, SentinelConfig, SentinelError};
use tokio::sync::mpsc::Receiver as MpscRx;

pub async fn eth_rpc_loop(mut eth_rpc_rx: MpscRx<EthRpcMessages>, config: SentinelConfig) -> Result<(), SentinelError> {
    let host_endpoints = config.get_host_endpoints();
    let native_endpoints = config.get_native_endpoints();

    'eth_rpc_loop: loop {
        tokio::select! {
            r = eth_rpc_rx.recv() => match r {
                Some(EthRpcMessages::GetLatestBlockNum((side, responder))) => {
                    let n = Ok(1337); // FIXME FIXME FIXME!
                    let _ = responder.send(n);
                    continue 'eth_rpc_loop
                },
                Some(msg) => {
                    break 'eth_rpc_loop Err(SentinelError::Custom(format!("handling {msg:?} not implemengted")))
                },
                None => {
                    let m = "all eth rpc senders dropped!";
                    warn!("{m}");
                    break 'eth_rpc_loop Err(SentinelError::Custom(m.into()))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("eth rpc shutting down...");
                break 'eth_rpc_loop Err(SentinelError::SigInt("eth rpc".into()))
            },
        }
    }
}
