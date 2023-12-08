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

    pub async fn rotate(&mut self) -> Result<WsClient, SentinelError> {
        self.increment_current_endpoint_index()?;
        info!("getting next endpoint @ index: {}", self.current);
        get_rpc_client(&self.endpoints[self.current]).await
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

    pub fn use_quicknode(&self) -> bool {
        // NOTE: TODO: Make more granular on a per endpoint basis. We use the `ws_client` directly
        // in a lot of places so we need to be able to get the URL from that (cannot as of yet) so
        // for now this quick and dirty method will have to do. It means we can only use quicknode
        // rpc calls if ALL endpoints strings are quicknode, so it's not ideal like this.
        // The other option was to ALWAYS try quicknode style RPC call first, but that's
        // inefficient too if interacting with a local node or a different service.
        self.endpoints
            .iter()
            .all(|s| s.contains("quiknode") || s.contains("quicknode"))
    }
}
