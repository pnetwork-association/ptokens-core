use std::result::Result;

use common::BridgeSide;
use lib::{get_latest_block_num, get_nonce, push_tx, EthRpcMessages, SentinelConfig, SentinelError};
use tokio::sync::mpsc::Receiver as MpscRx;

pub async fn eth_rpc_loop(mut eth_rpc_rx: MpscRx<EthRpcMessages>, config: SentinelConfig) -> Result<(), SentinelError> {
    let host_endpoints = config.get_host_endpoints();
    let native_endpoints = config.get_native_endpoints();

    'eth_rpc_loop: loop {
        tokio::select! {
            r = eth_rpc_rx.recv() => match r {
                Some(EthRpcMessages::GetLatestBlockNum((side, responder))) => {
                    let r = match side {
                        BridgeSide::Host => get_latest_block_num(&host_endpoints),
                        BridgeSide::Native => get_latest_block_num(&native_endpoints),
                    }.await;
                    let _ = responder.send(r);
                    continue 'eth_rpc_loop
                },
                Some(EthRpcMessages::PushTx((tx, side, responder))) => {
                    let r = match side {
                        BridgeSide::Host => push_tx(tx, &host_endpoints),
                        BridgeSide::Native => push_tx(tx, &native_endpoints),
                    }.await;
                    let _ = responder.send(r);
                    continue 'eth_rpc_loop
                },
                Some(EthRpcMessages::GetNonce((side, address, responder))) => {
                    let r = match side {
                        BridgeSide::Host => get_nonce(&host_endpoints, &address),
                        BridgeSide::Native => get_nonce(&native_endpoints, &address),
                    }.await;
                    let _ = responder.send(r);
                    continue 'eth_rpc_loop
                }
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
