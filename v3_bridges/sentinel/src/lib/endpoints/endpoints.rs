use std::result::Result;

use common::BridgeSide;
use derive_more::Constructor;
use jsonrpsee::ws_client::WsClient;

use crate::{config::ConfigError, get_rpc_client, SentinelError};

#[derive(Debug, Default, Clone)]
pub struct Endpoints {
    is_native: bool,
    sleep_time: u64,
    side: BridgeSide,
    endpoints: Vec<String>,
}

impl Endpoints {
    pub fn new(is_native: bool, sleep_time: u64, side: BridgeSide, endpoints: Vec<String>) -> Self {
        Self { is_native, sleep_time, side, endpoints }
    }

    pub fn get_first_endpoint(&self) -> Result<String, SentinelError> {
        let endpoint_type = if self.is_native { "native" } else { "host" };
        info!("[+] Getting first {endpoint_type} endpoint...");
        if self.endpoints.is_empty() {
            Err(ConfigError::NoEndpoints(self.side).into())
        } else {
            Ok(self.endpoints[0].clone())
        }
    }

    pub async fn get_rpc_client(&self) -> Result<WsClient, SentinelError> {
        let endpoint = self.get_first_endpoint()?;
        let rpc_client = get_rpc_client(&endpoint).await?;
        Ok(rpc_client)
    }

    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    pub fn side(&self) -> BridgeSide {
        self.side
    }
}
