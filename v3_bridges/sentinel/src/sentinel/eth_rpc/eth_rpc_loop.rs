use std::result::Result;

use common::BridgeSide;
use lib::{
    eth_call,
    get_gas_price,
    get_latest_block_num,
    get_nonce,
    push_tx,
    EthRpcMessages,
    SentinelConfig,
    SentinelError,
};
use tokio::sync::mpsc::Receiver as MpscRx;

pub async fn eth_rpc_loop(mut eth_rpc_rx: MpscRx<EthRpcMessages>, config: SentinelConfig) -> Result<(), SentinelError> {
    let host_endpoints = config.get_host_endpoints();
    let native_endpoints = config.get_native_endpoints();
    let h_ws_client = host_endpoints.get_ws_client().await?;
    let n_ws_client = native_endpoints.get_ws_client().await?;

    'eth_rpc_loop: loop {
        tokio::select! {
            r = eth_rpc_rx.recv() => match r {
                Some(EthRpcMessages::GetLatestBlockNum((side, responder))) => {
                    let r = match side {
                        BridgeSide::Host => get_latest_block_num(&h_ws_client),
                        BridgeSide::Native => get_latest_block_num(&n_ws_client),
                    }.await;
                    let _ = responder.send(r);
                    continue 'eth_rpc_loop
                },
                Some(EthRpcMessages::GetGasPrice((side, responder))) => {
                    let r = match side {
                        BridgeSide::Host => get_gas_price(&h_ws_client),
                        BridgeSide::Native => get_gas_price(&n_ws_client),
                    }.await;
                    let _ = responder.send(r);
                    continue 'eth_rpc_loop
                },
                Some(EthRpcMessages::PushTx((tx, side, responder))) => {
                    let r = match side {
                        BridgeSide::Host => push_tx(tx, &h_ws_client),
                        BridgeSide::Native => push_tx(tx, &n_ws_client),
                    }.await;
                    let _ = responder.send(r);
                    continue 'eth_rpc_loop
                },
                Some(EthRpcMessages::GetNonce((side, address, responder))) => {
                    let r = match side {
                        BridgeSide::Host => get_nonce(&h_ws_client, &address),
                        BridgeSide::Native => get_nonce(&n_ws_client, &address),
                    }.await;
                    let _ = responder.send(r);
                    continue 'eth_rpc_loop
                },
                Some(EthRpcMessages::EthCall((data, side, address, default_block_parameter, responder))) => {
                    let r = match side {
                        BridgeSide::Host => eth_call(&address, &data, &default_block_parameter, &h_ws_client),
                        BridgeSide::Native => eth_call(&address, &data, &default_block_parameter, &n_ws_client),
                    }.await;
                    let _ = responder.send(r);
                    continue 'eth_rpc_loop
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