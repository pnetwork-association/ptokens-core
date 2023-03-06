use anyhow::Result;
use derive_more::Constructor;
use jsonrpsee::ws_client::WsClient;

use crate::{check_endpoint, get_rpc_client};

#[derive(Debug, Default, Clone, Eq, PartialEq, Constructor)]
pub struct Endpoints {
    is_native: bool,
    sleep_time: u64,
    endpoints: Vec<String>,
}

impl Endpoints {
    pub fn get_first_endpoint(&self) -> Result<String> {
        let endpoint_type = if self.is_native { "native" } else { "host" };
        info!("[+] Getting first {endpoint_type} endpoint...");
        if self.endpoints.is_empty() {
            Err(anyhow!("No {endpoint_type} endpoints in config file!"))
        } else {
            Ok(self.endpoints[0].clone())
        }
    }

    pub async fn get_rpc_client(&self) -> Result<WsClient> {
        let endpoint = self.get_first_endpoint()?;
        let rpc_client = get_rpc_client(&endpoint).await?;
        check_endpoint(&rpc_client, self.sleep_time).await?;
        Ok(rpc_client)
    }

    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }
}
