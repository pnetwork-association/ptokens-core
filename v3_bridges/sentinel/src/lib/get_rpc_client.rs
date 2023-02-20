use anyhow::Result;
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};

pub async fn get_rpc_client(url: &str) -> Result<WsClient> {
    info!("[+] Getting RPC client...");
    Ok(WsClientBuilder::default().build(&url).await?)
}
