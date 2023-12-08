use std::result::Result;

use common_sentinel::{
    eth_call,
    get_challenge_state,
    get_eth_balance,
    get_gas_price,
    get_latest_block_num,
    get_nonce,
    get_sub_mat,
    get_user_op_state,
    push_tx,
    BroadcastChannelMessages,
    EthRpcMessages,
    NetworkId,
    SentinelConfig,
    SentinelError,
};
use tokio::{
    sync::{
        broadcast::{Receiver as MpMcRx, Sender as MpMcTx},
        mpsc::Receiver as MpscRx,
    },
    time::{sleep, Duration},
};

// NOTE: The underlying RPC calls have both retry & timeout logic, however in the event of a websocket disconnect, they
// immediately return with an error. That error is handled in each of the arms below, via rotating the endpoint to get a
// new socket.

// TODO DRY out the repeat code below, though it's not trivial due to having to replace the mutable websocket clients
// upon endpoint rotation.

const ENDPOINT_ROTATION_SLEEP_TIME: u64 = 20;

pub async fn eth_rpc_loop(
    mut eth_rpc_rx: MpscRx<EthRpcMessages>,
    config: SentinelConfig,
    network_id: NetworkId,
    _broadcast_channel_tx: MpMcTx<BroadcastChannelMessages>,
    _broadcast_channel_rx: MpMcRx<BroadcastChannelMessages>,
) -> Result<(), SentinelError> {
    let mut endpoints = config.endpoints(&network_id)?;
    let use_quicknode = endpoints.use_quicknode();
    let sleep_duration = *endpoints.sleep_time();
    let mut ws_client = endpoints.get_first_ws_client().await?;

    'eth_rpc_loop: loop {
        tokio::select! {
            r = eth_rpc_rx.recv() => match r {
                Some(msg) => {
                    match msg {
                        EthRpcMessages::GetChallengeState((network_id, challenge, pnetwork_hub, responder)) => {
                            'inner: loop {
                                let r = get_challenge_state(
                                    &challenge,
                                    &pnetwork_hub,
                                    &ws_client,
                                    sleep_duration,
                                    network_id,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetUserOpState((network_id, user_op, contract_address, responder)) => {
                            'inner: loop {
                                let r = get_user_op_state(
                                    &user_op,
                                    &contract_address,
                                    &ws_client,
                                    sleep_duration,
                                    network_id,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetLatestBlockNum((network_id, responder)) => {
                            'inner: loop {
                                let r = get_latest_block_num(
                                    &ws_client,
                                    sleep_duration,
                                    &network_id,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetGasPrice((network_id, responder)) => {
                            'inner: loop {
                                let r = get_gas_price(
                                    &ws_client,
                                    sleep_duration,
                                    network_id,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::PushTx((tx, network_id, responder)) => {
                            'inner: loop {
                                let r = push_tx(
                                    &tx,
                                    &ws_client,
                                    sleep_duration,
                                    &network_id,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetNonce((network_id, address, responder)) => {
                            'inner: loop {
                                let r = get_nonce(
                                    &ws_client,
                                    &address,
                                    sleep_duration,
                                    network_id,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::EthCall((data, network_id, address, default_block_parameter, responder)) => {
                            'inner: loop {
                                let r = eth_call(
                                    &address,
                                    &data,
                                    &default_block_parameter,
                                    &ws_client,
                                    sleep_duration,
                                    network_id,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    },
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetSubMat((network_id, block_num, responder)) => {
                            'inner: loop {
                                let r = get_sub_mat(
                                    &ws_client,
                                    block_num,
                                    sleep_duration,
                                    &network_id,
                                    use_quicknode,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    }
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                        EthRpcMessages::GetEthBalance((network_id, address, responder)) => {
                            'inner: loop {
                                let r = get_eth_balance(
                                    &ws_client,
                                    &address,
                                    sleep_duration,
                                    network_id,
                                ).await;
                                match r {
                                    Ok(r) => {
                                        let _ = responder.send(Ok(r));
                                        continue 'eth_rpc_loop
                                    }
                                    Err(e) => {
                                        error!("{network_id} eth rpc error: {e}");
                                        warn!("rotating {network_id} endpoint");
                                        sleep(Duration::from_secs(ENDPOINT_ROTATION_SLEEP_TIME)).await;
                                        ws_client = endpoints.rotate().await?;
                                        continue 'inner
                                    },
                                }
                            }
                        },
                    }
                },
                None => {
                    let m = format!("all eth rpc for network {network_id} senders dropped!");
                    error!("{m}");
                    break 'eth_rpc_loop Err(SentinelError::Custom(m))
                },
            },
            _ = tokio::signal::ctrl_c() => {
                warn!("eth rpc for network {network_id} shutting down...");
                break 'eth_rpc_loop Err(SentinelError::SigInt("eth rpc".into()))
            },
        }
    }
}
