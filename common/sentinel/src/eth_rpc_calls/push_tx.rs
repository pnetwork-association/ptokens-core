use std::result::Result;

use common::BridgeSide;
use common_eth::{convert_hex_to_h256, EthTransaction};
use ethereum_types::H256;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};
use tokio::time::{sleep, Duration};

use super::{ETH_RPC_CALL_TIME_LIMIT, MAX_RPC_CALL_ATTEMPTS};
use crate::{run_timer, EndpointError, SentinelError};

const RPC_CMD: &str = "eth_sendRawTransaction";

async fn push_tx_inner(tx: &EthTransaction, ws_client: &WsClient) -> Result<H256, SentinelError> {
    let res: Result<String, jsonrpsee::core::Error> = ws_client
        .request(RPC_CMD, rpc_params![format!("0x{}", tx.serialize_hex())])
        .await;
    match res {
        Ok(ref s) => Ok(convert_hex_to_h256(s)?),
        Err(e) => Err(EndpointError::PushTx(e).into()),
    }
}

pub async fn push_tx(
    tx: &EthTransaction,
    ws_client: &WsClient,
    sleep_time: u64,
    side: BridgeSide,
) -> Result<H256, SentinelError> {
    let mut attempt = 1;
    loop {
        let m = format!("{side} pushing tx attempt #{attempt}");
        debug!("{m}");

        let r = tokio::select! {
            res = push_tx_inner(tx, ws_client) => res,
            _ = run_timer(ETH_RPC_CALL_TIME_LIMIT) => Err(EndpointError::TimeOut(m.clone()).into()),
            _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m.clone()).into()),
        };

        match r {
            Ok(r) => break Ok(r),
            Err(e) => match e {
                SentinelError::Endpoint(EndpointError::WsClientDisconnected(_)) => {
                    warn!("{side} {RPC_CMD} failed due to web socket dropping");
                    break Err(e);
                },
                _ => {
                    if attempt < MAX_RPC_CALL_ATTEMPTS {
                        attempt += 1;
                        error!("{e}");
                        warn!("{side} sleeping for {sleep_time}ms before retrying...");
                        sleep(Duration::from_millis(sleep_time)).await;
                        continue;
                    } else {
                        warn!("{side} {RPC_CMD} failed after {attempt} attempts");
                        break Err(e);
                    }
                },
            },
        }
    }
}
