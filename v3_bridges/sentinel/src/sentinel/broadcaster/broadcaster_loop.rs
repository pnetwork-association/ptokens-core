use std::{convert::TryFrom, result::Result};

use common::BridgeSide;
use common_chain_ids::EthChainId;
use common_eth::{EthPrivateKey, EthTransaction};
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::Address as EthAddress;
use lib::{BroadcasterMessages, EthRpcMessages, MongoMessages, SentinelConfig, SentinelError};
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

fn get_tx(nonce: u64, chain_id: &EthChainId) -> Result<EthTransaction, SentinelError> {
    let value = 0;
    let gas_limt = 1_000_000;
    // TODO handle gas price properly
    let gas_price = 2_000_000_000;
    /*
    EthTransaction::new_unsigned(
        data: Bytes,
        nonce: u64,
        value: usize,
        to: EthAddress,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
    ).sign(pk)

    encode_fxn_call(ERC20_VAULT_ABI, "pegOut", &[
        EthAbiToken::Address(recipient),
        EthAbiToken::Address(token_contract_address),
        EthAbiToken::Uint(amount),
    ])

     */
    Ok(EthTransaction::default())
}

async fn main_loop(
    config: &SentinelConfig,
    mut mongo_tx: MpscTx<MongoMessages>,
    mut eth_rpc_tx: MpscTx<EthRpcMessages>,
) -> Result<(), SentinelError> {
    let mut debug_loop_counter = 0;
    let sleep_duration = config.mongo_config.sleep_duration();
    let address = get_eth_address()?;
    let (n_msg, n_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Native, address);
    let (h_msg, h_rx) = EthRpcMessages::get_nonce_msg(BridgeSide::Native, address);
    eth_rpc_tx.send(n_msg).await?;
    eth_rpc_tx.send(h_msg).await?;

    let mut n_nonce = n_rx.await??;
    let mut h_nonce = h_rx.await??;

    let n_chain_id = config.native_config.get_eth_chain_id();
    let h_chain_id = config.host_config.get_eth_chain_id();

    'broadcaster_loop: loop {
        let (msg, rx) = MongoMessages::get_output_msg();
        mongo_tx.send(msg).await?;
        let output = rx.await??;

        let h_unmatched = output.host_unmatched_user_ops();
        let n_unmatched = output.host_unmatched_user_ops();

        if !n_unmatched.is_empty() {
            // TODO this could need multiple txs?
            let eth_tx = get_tx(n_nonce, &n_chain_id)?;
            let (msg, rx) = EthRpcMessages::get_push_tx_msg(eth_tx, BridgeSide::Native);
            eth_rpc_tx.send(msg).await?;
            let tx_hash = rx.await??;
            n_nonce += 1;
            debug!("native tx pushed: {tx_hash}");
        }

        if !h_unmatched.is_empty() {
            // TODO this could need multiple txs?
            let eth_tx = get_tx(h_nonce, &h_chain_id)?;
            let (msg, rx) = EthRpcMessages::get_push_tx_msg(eth_tx, BridgeSide::Host);
            eth_rpc_tx.send(msg).await?;
            let tx_hash = rx.await??;
            h_nonce += 1;
            debug!("host tx pushed: {tx_hash}");
        }

        sleep(Duration::from_millis(sleep_duration)).await;
        debug_loop_counter += 1;
        continue 'broadcaster_loop;
    }
}

pub async fn broadcaster_loop(
    rx: MpscRx<BroadcasterMessages>,
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
                /*
                if r.is_ok() {
                    continue
                } else {
                    break r
                }
                */
            _ = tokio::signal::ctrl_c() => {
                warn!("broadcaster shutting down...");
                Err(SentinelError::SigInt("broadcaster".into()))
            },
        }
    }
}
