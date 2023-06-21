use std::result::Result;

use common_eth::{convert_hex_to_h256, EthTransaction};
use ethereum_types::H256;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

use super::{ETH_RPC_CALL_TIME_LIMIT, MAX_RPC_CALL_ATTEMPTS};
use crate::{run_timer, EndpointError, SentinelError};

const PUSH_TX_RPC_CMD: &str = "eth_sendRawTransaction";

async fn push_tx_inner(tx: &EthTransaction, ws_client: &WsClient) -> Result<H256, SentinelError> {
    let res: Result<String, jsonrpsee::core::Error> = ws_client
        .request(PUSH_TX_RPC_CMD, rpc_params![tx.serialize_hex()])
        .await;
    match res {
        Ok(ref s) => Ok(convert_hex_to_h256(s)?),
        Err(e) => Err(EndpointError::PushTx(e).into()),
    }
}

pub async fn push_tx(tx: EthTransaction, ws_client: &WsClient) -> Result<H256, SentinelError> {
    let mut attempt = 1;
    let m = format!("pushing tx attempt #{attempt}");
    debug!("{m}");

    loop {
        let r = tokio::select! {
            res = push_tx_inner(&tx, ws_client) => res,
            _ = run_timer(ETH_RPC_CALL_TIME_LIMIT) => Err(EndpointError::TimeOut(m.clone()).into()),
            _ = ws_client.on_disconnect() => Err(EndpointError::WsClientDisconnected(m.clone()).into()),
        };

        match r {
            Ok(r) => break Ok(r),
            Err(e) => match e {
                SentinelError::Endpoint(EndpointError::WsClientDisconnected(_)) => {
                    break Err(e);
                },
                _ => {
                    if attempt < MAX_RPC_CALL_ATTEMPTS {
                        attempt += 1;
                        continue;
                    } else {
                        break Err(e);
                    }
                },
            },
        }
    }
}
