use std::result::Result;

use common::BridgeSide;
use ethereum_types::{Address as EthAddress, H256 as EthHash};
use lib::{
    BroadcasterMessages,
    ConfigT,
    CoreMessages,
    EthRpcMessages,
    MongoMessages,
    SentinelConfig,
    SentinelError,
    UserOp,
};
use tokio::sync::mpsc::{Receiver as MpscRx, Sender as MpscTx};

async fn cancel_user_op(
    op: UserOp,
    nonce: u64,
    gas_price: u64,
    gas_limit: usize,
    core_tx: MpscTx<CoreMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    pnetwork_hub: &EthAddress,
) -> Result<EthHash, SentinelError> {
    // TODO check we have enough balance to push
    // TODO put back in core db upon error and continue broadcaster loop with warning messages?

    let side = op.destination_side();
    debug!("cancelling user op on side: {side} nonce: {nonce} gas price: {gas_price}");

    let (msg, rx) = EthRpcMessages::get_user_op_state_msg(side, op.clone(), *pnetwork_hub);
    eth_rpc_tx.send(msg).await?;
    let user_op_smart_contract_state = rx.await??;
    debug!("user op state before cancellation: {user_op_smart_contract_state}");

    let tx_hash = if user_op_smart_contract_state.is_cancellable() {
        warn!("sending cancellation tx for user op: {op}");
        let (msg, rx) =
            CoreMessages::get_cancellation_signature_msg(op.clone(), nonce, gas_price, gas_limit, *pnetwork_hub);
        core_tx.send(msg).await?;
        let signed_tx = rx.await??;
        debug!("signed tx: {}", signed_tx.serialize_hex());

        let (msg, rx) = EthRpcMessages::get_push_tx_msg(signed_tx, side);
        eth_rpc_tx.send(msg).await?;
        let tx_hash = rx.await??;
        info!("tx hash: {tx_hash}");
        tx_hash
    } else {
        EthHash::zero()
    };

    Ok(tx_hash)
}

async fn get_gas_price(config: &impl ConfigT, eth_rpc_tx: MpscTx<EthRpcMessages>) -> Result<u64, SentinelError> {
    let side = config.side();
    let p = if let Some(p) = config.gas_price() {
        debug!("using {side} gas price from config: {p}");
        p
    } else {
        let (msg, rx) = EthRpcMessages::get_gas_price_msg(side);
        eth_rpc_tx.send(msg).await?;
        let p = rx.await??;
        debug!("using {side} gas price from rpc: {p}");
        p
    };
    Ok(p)
}

async fn cancel_user_ops(
    config: &SentinelConfig,
    host_address: &EthAddress,
    native_address: &EthAddress,
    core_tx: MpscTx<CoreMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
) -> Result<(), SentinelError> {
    let host_pnetwork_hub = config.host().pnetwork_hub();
    let native_pnetwork_hub = config.native().pnetwork_hub();

    let (host_msg, host_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Host, *host_address);
    eth_rpc_tx.send(host_msg).await?;
    let mut host_nonce = host_rx.await??;

    let (native_msg, native_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Native, *native_address);
    eth_rpc_tx.send(native_msg).await?;
    let mut native_nonce = native_rx.await??;

    let host_gas_price = get_gas_price(config.host(), eth_rpc_tx.clone()).await?;
    let native_gas_price = get_gas_price(config.native(), eth_rpc_tx.clone()).await?;

    let host_gas_limit = config.host().gas_limit();
    let native_gas_limit = config.native().gas_limit();

    let (cancellable_ops_msg, cancellable_ops_rx) = CoreMessages::get_cancellable_user_ops_msg();
    core_tx.send(cancellable_ops_msg).await?;
    let cancellable_user_ops = cancellable_ops_rx.await??;

    let err_msg = "error cancelling user op ";

    for op in cancellable_user_ops.iter() {
        match op.destination_side() {
            BridgeSide::Native => {
                let uid = op.uid()?;
                match cancel_user_op(
                    op.clone(),
                    native_nonce,
                    native_gas_price,
                    native_gas_limit,
                    core_tx.clone(),
                    eth_rpc_tx.clone(),
                    &native_pnetwork_hub,
                )
                .await
                {
                    Err(e) => {
                        error!("{err_msg} {uid} {e}");
                    },
                    Ok(tx_hash) => {
                        info!(
                            "user op {uid} cancelled successfully @ tx {}",
                            hex::encode(tx_hash.as_bytes())
                        );
                    },
                }
                native_nonce += 1;
            },
            BridgeSide::Host => {
                let uid = op.uid()?;
                match cancel_user_op(
                    op.clone(),
                    host_nonce,
                    host_gas_price,
                    host_gas_limit,
                    core_tx.clone(),
                    eth_rpc_tx.clone(),
                    &host_pnetwork_hub,
                )
                .await
                {
                    Err(e) => {
                        error!("{err_msg} {uid} {e}");
                    },
                    Ok(tx_hash) => {
                        info!(
                            "user op {uid} cancelled successfully @ tx {}",
                            hex::encode(tx_hash.as_bytes())
                        );
                    },
                }
                host_nonce += 1;
            },
        }
    }

    Ok(())
}

pub async fn broadcaster_loop(
    mut rx: MpscRx<BroadcasterMessages>,
    _mongo_tx: MpscTx<MongoMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    core_tx: MpscTx<CoreMessages>,
    config: SentinelConfig,
    disable_broadcaster: bool,
) -> Result<(), SentinelError> {
    if disable_broadcaster {
        warn!("Broadcaster has been disabled");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                warn!("broadcaster shutting down...");
                Err(SentinelError::SigInt("broadcaster".into()))
            },
        }
    } else {
        // TODO could get all the signing params fom the core here and collection them into a struct?
        let (host_msg, host_rx) = CoreMessages::get_address_msg(BridgeSide::Host);
        let (native_msg, native_rx) = CoreMessages::get_address_msg(BridgeSide::Native);
        core_tx.send(host_msg).await?;
        core_tx.send(native_msg).await?;
        let host_address = host_rx.await??;
        let native_address = native_rx.await??;

        debug!("Broadcaster loop running...");
        'broadcaster_loop: loop {
            tokio::select! {
                r = rx.recv() => match r {
                    Some(BroadcasterMessages::CancelUserOps) => {
                        match cancel_user_ops(
                            &config,
                            &host_address,
                            &native_address,
                            core_tx.clone(),
                            eth_rpc_tx.clone()
                        ).await {
                            Ok(_) => {
                                info!("finished sending user op cancellation txs");
                            }
                            Err(e) => {
                                error!("{e}");
                            }
                        };
                        continue 'broadcaster_loop
                    },
                    None => {
                        let m = "all broadcaster senders dropped!";
                        warn!("{m}");
                        break 'broadcaster_loop Err(SentinelError::Custom(m.into()))
                    },
                },
                _ = tokio::signal::ctrl_c() => {
                    warn!("broadcaster shutting down...");
                    return Err(SentinelError::SigInt("broadcaster".into()))
                },
            }
        }
    }
}
