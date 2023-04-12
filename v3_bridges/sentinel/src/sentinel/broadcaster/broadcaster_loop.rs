use std::result::Result;

use common::BridgeSide;
use lib::{BroadcasterMessages, EthRpcMessages, MongoMessages, SentinelConfig, SentinelError};
use tokio::{
    sync::mpsc::{Receiver as MpscRx, Sender as MpscTx},
    time::{sleep, Duration},
};

async fn main_loop(
    config: &SentinelConfig,
    _mongo_tx: MpscTx<MongoMessages>,
    eth_rpc_tx: MpscTx<EthRpcMessages>,
) -> Result<(), SentinelError> {
    let mut debug_loop_counter = 0;

    let sleep_duration = config.mongo().sleep_duration();
    let address = common_eth::convert_hex_to_eth_address("edB86cd455ef3ca43f0e227e00469C3bDFA40628")?; // FIXME
    let (n_msg, n_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Native, address);
    let (h_msg, h_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Host, address);
    eth_rpc_tx.send(n_msg).await?;
    eth_rpc_tx.send(h_msg).await?;

    let _n_nonce = n_rx.await??;
    let _h_nonce = h_rx.await??;

    let _n_chain_id = config.native().get_eth_chain_id();
    let _h_chain_id = config.host().get_eth_chain_id();

    let _n_state_manager = config.state_manager(&BridgeSide::Native);
    let _h_state_manager = config.state_manager(&BridgeSide::Host);

    'broadcaster_loop: loop {
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
