use std::result::Result;

use common_eth::{convert_hex_to_h256, EthTransaction};
use ethereum_types::H256;
use jsonrpsee::{core::client::ClientT, rpc_params};

use crate::{EndpointError, Endpoints, SentinelError};

const PUSH_TX_RPC_CMD: &str = "eth_sendRawTransaction";

pub async fn push_tx(tx: EthTransaction, endpoints: &Endpoints) -> Result<H256, SentinelError> {
    debug!("Pushing {} tx...", endpoints.side());
    let client = endpoints.get_rpc_client().await?;
    let res: jsonrpsee::core::RpcResult<String> =
        client.request(PUSH_TX_RPC_CMD, rpc_params![tx.serialize_hex()]).await;
    match res {
        Ok(ref s) => Ok(convert_hex_to_h256(s)?),
        Err(e) => Err(SentinelError::Endpoint(EndpointError::PushTx(e))),
    }
}
