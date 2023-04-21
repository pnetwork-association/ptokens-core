use std::result::Result;

use common_eth::{convert_hex_to_h256, EthTransaction};
use ethereum_types::H256;
use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClient};

use crate::{EndpointError, SentinelError};

const PUSH_TX_RPC_CMD: &str = "eth_sendRawTransaction";

pub async fn push_tx(tx: EthTransaction, ws_client: &WsClient) -> Result<H256, SentinelError> {
    debug!("Pushing tx...");
    let res: jsonrpsee::core::RpcResult<String> = ws_client
        .request(PUSH_TX_RPC_CMD, rpc_params![tx.serialize_hex()])
        .await;
    match res {
        Ok(ref s) => Ok(convert_hex_to_h256(s)?),
        Err(e) => Err(EndpointError::PushTx(e).into()),
    }
}
