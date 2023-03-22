use std::result::Result;

use common::BridgeSide;
use lib::{get_latest_block_num_via_endpoints, EthRpcMessages, SentinelConfig, SentinelError};
use tokio::sync::mpsc::Receiver as MpscRx;

pub async fn eth_rpc_loop(mut eth_rpc_rx: MpscRx<EthRpcMessages>, config: SentinelConfig) -> Result<(), SentinelError> {
    let host_endpoints = config.get_host_endpoints();
    let native_endpoints = config.get_native_endpoints();

    'eth_rpc_loop: loop {
        tokio::select! {
            r = eth_rpc_rx.recv() => match r {
                Some(EthRpcMessages::GetLatestBlockNum((side, responder))) => {
                    let r = match side {
                        BridgeSide::Host => get_latest_block_num_via_endpoints(&host_endpoints),
                        BridgeSide::Native => get_latest_block_num_via_endpoints(&native_endpoints),
                    }.await;
                    let _ = responder.send(r);
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
