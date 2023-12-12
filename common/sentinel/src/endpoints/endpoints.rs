use std::result::Result;

use derive_getters::Getters;
use jsonrpsee::ws_client::WsClient;

use super::EndpointError;
use crate::{get_rpc_client, NetworkId, SentinelConfigError, SentinelError};

#[derive(Debug, Default, Clone, Getters)]
pub struct Endpoints {
    current: usize,
    sleep_time: u64,
    rotations: usize,
    network_id: NetworkId,
    endpoints: Vec<String>,
}

impl Endpoints {
    pub fn use_quicknode(&self) -> bool {
        let ce = self.current_endpoint();
        if ce.contains("goerli") || ce.contains("sepolia") || ce.contains("xdai") {
            // NOTE: The xdai one is a temporary restriction because their `qn_` endpoints are
            // currently not working correctly.
            // NOTE: Quicknode does not offer their single flight endpoints for testnet chains.
            // https://marketplace.quicknode.com/add-on/single-flight-rpc
            false
        } else {
            ce.contains("quiknode") || ce.contains("quicknode")
        }
    }

    pub fn new(sleep_time: u64, network_id: NetworkId, endpoints: Vec<String>) -> Self {
        Self {
            endpoints,
            network_id,
            sleep_time,
            ..Default::default()
        }
    }

    fn get_first_endpoint(&self) -> Result<String, SentinelConfigError> {
        info!("getting first endpoint for network {}", self.network_id());
        if self.endpoints.is_empty() {
            Err(SentinelConfigError::NoEndpoints(self.network_id))
        } else {
            Ok(self.endpoints[0].clone())
        }
    }

    pub async fn get_first_ws_client(&self) -> Result<WsClient, SentinelError> {
        let endpoint = self.get_first_endpoint()?;
        let rpc_client = get_rpc_client(&endpoint).await?;
        Ok(rpc_client)
    }

    fn current_endpoint(&self) -> &str {
        &self.endpoints[self.current]
    }

    pub async fn rotate(&mut self) -> Result<WsClient, SentinelError> {
        self.increment_current_endpoint_index()?;
        info!("getting next endpoint @ index: {}", self.current);
        get_rpc_client(self.current_endpoint()).await
    }

    fn increment_current_endpoint_index(&mut self) -> Result<(), EndpointError> {
        let next = (self.current + 1) % self.endpoints.len();
        debug!(
            "increment endpoint index from {} to {next} (num endpoints: {})",
            self.current,
            self.endpoints.len()
        );
        if next == 0 {
            self.rotations += 1;
            debug!("incrementing num rotationsi to {}", self.rotations);
        }
        warn!(
            "on endpoint rotation #{} for network {}",
            self.network_id(),
            self.rotations
        );
        self.current = next;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }
}
