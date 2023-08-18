use std::result::Result;

use jsonrpsee::ws_client::{WsClient, WsClientBuilder};

use crate::SentinelError;

pub async fn get_rpc_client(url: &str) -> Result<WsClient, SentinelError> {
    debug!("Getting RPC client using URL '{url}'...");
    Ok(WsClientBuilder::default().build(&url).await?)
}
