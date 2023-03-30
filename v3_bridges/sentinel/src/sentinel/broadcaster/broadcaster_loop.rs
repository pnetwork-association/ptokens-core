use std::{convert::TryFrom, result::Result};

use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_eth::{EthPrivateKey, EthTransaction};
use ethereum_types::Address as EthAddress;
use lib::{BroadcasterMessages, EthRpcMessages, MongoMessages, SentinelConfig, SentinelError, UserOperation};
use tokio::{
    sync::mpsc::{Receiver as MpscRx, Sender as MpscTx},
    time::{sleep, Duration},
};

fn get_pk() -> Result<EthPrivateKey, SentinelError> {
    // TODO Handle pk properly
    let testnet_pk_hex = "64aaa58f496810ef053e25a734d1fbd90ddf5d33838bb3700014ceb59ca3204d";
    Ok(EthPrivateKey::try_from(testnet_pk_hex)?)
}

fn get_eth_address() -> Result<EthAddress, SentinelError> {
    get_pk().map(|pk| pk.to_public_key().to_address())
}

fn get_tx(
    nonce: u64,
    chain_id: &EthChainId,
    state_manager: &EthAddress,
    op: &UserOperation,
) -> Result<EthTransaction, SentinelError> {
    let value = 0;
    let gas_limit = 1_000_000; // FIXME
    let gas_price = 2_000_000_000; // FIXME
    let to = *state_manager;
    let data = op.to_cancel_fxn_data()?; //TODO make fxn to convert user op to a cancelling EthTransaction type.

    Ok(EthTransaction::new_unsigned(data, nonce, value, to, chain_id, gas_limit, gas_price).sign(&get_pk()?)?)
}

async fn main_loop(
    config: &SentinelConfig,
    mongo_tx: MpscTx<MongoMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
) -> Result<(), SentinelError> {
    let mut debug_loop_counter = 0;

    let sleep_duration = config.mongo_config.sleep_duration();
    let address = get_eth_address()?;
    let (n_msg, n_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Native, address);
    let (h_msg, h_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Host, address);
    eth_rpc_tx.send(n_msg).await?;
    eth_rpc_tx.send(h_msg).await?;

    let mut n_nonce = n_rx.await??;
    let mut h_nonce = h_rx.await??;

    let n_chain_id = config.native_config.get_eth_chain_id();
    let h_chain_id = config.host_config.get_eth_chain_id();

    let n_state_manager = config.get_state_manager(&BridgeSide::Native);
    let h_state_manager = config.get_state_manager(&BridgeSide::Host);

    'broadcaster_loop: loop {
        let (msg, rx) = MongoMessages::get_output_msg();
        mongo_tx.send(msg).await?;
        let output = rx.await??;

        let h_unmatched = output.host_unmatched_user_ops();
        let n_unmatched = output.host_unmatched_user_ops();

        if !n_unmatched.is_empty() {
            // TODO this could need multiple txs?
            // TODO checks to see if we need to even send a tx?
            let eth_tx = get_tx(n_nonce, &n_chain_id, &n_state_manager, &n_unmatched[0])?;
            let (msg, rx) = EthRpcMessages::get_push_tx_msg(eth_tx, BridgeSide::Native);
            eth_rpc_tx.send(msg).await?;
            let tx_hash = rx.await??;
            n_nonce += 1;
            //n_unmatched = UserOperations::new(n_unmatched[1..].to_vec()); TODO update in the DB!
            debug!("native tx pushed: {tx_hash}");
        }

        if !h_unmatched.is_empty() {
            // TODO this could need multiple txs?
            // TODO cheks that we even need to send a tx
            let eth_tx = get_tx(h_nonce, &h_chain_id, &h_state_manager, &h_unmatched[0])?;
            let (msg, rx) = EthRpcMessages::get_push_tx_msg(eth_tx, BridgeSide::Host);
            eth_rpc_tx.send(msg).await?;
            let tx_hash = rx.await??;
            h_nonce += 1;
            //h_unmatched = UserOperations::new(h_unmatched[1..].to_vec()); // TODO update in the DB
            debug!("host tx pushed: {tx_hash}");
        }

        sleep(Duration::from_millis(sleep_duration)).await;
        debug_loop_counter += 1;
        debug!("broadcaster loop count: {debug_loop_counter}");
        continue 'broadcaster_loop;
    }
}

pub async fn broadcaster_loop(
    _rx: MpscRx<BroadcasterMessages>,
    mongo_tx: MpscTx<MongoMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
    config: SentinelConfig,
    disable_broadcaster: bool,
) -> Result<(), SentinelError> {
    let _host_endpoints = config.get_host_endpoints();
    let _native_endpoints = config.get_native_endpoints();

    if disable_broadcaster {
        warn!("Broadcaster has been disabled");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                warn!("broadcaster shutting down...");
                Err(SentinelError::SigInt("broadcaster".into()))
            },
        }
    } else {
        debug!("Broadcaster loop running...");
        tokio::select! {
            r = main_loop(&config, mongo_tx, eth_rpc_tx) => r,
            _ = tokio::signal::ctrl_c() => {
                warn!("broadcaster shutting down...");
                Err(SentinelError::SigInt("broadcaster".into()))
            },
        }
    }
}
