use anyhow::Result;
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};

use crate::check_endpoint;

pub async fn get_rpc_client(url: &str) -> Result<WsClient> {
    info!("[+] Getting RPC client...");
    let ws_client = WsClientBuilder::default().build(&url).await?;
    check_endpoint(&ws_client).await?;
    Ok(ws_client)
}
