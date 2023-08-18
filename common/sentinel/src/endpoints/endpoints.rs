use std::result::Result;

use common::BridgeSide;
use jsonrpsee::ws_client::WsClient;

use super::EndpointError;
use crate::{get_rpc_client, SentinelConfigError, SentinelError};

#[derive(Debug, Default, Clone)]
pub struct Endpoints {
    sleep_time: u64,
    side: BridgeSide,
    endpoints: Vec<String>,
    current: usize,
    rotations: usize,
}

impl Endpoints {
    pub fn new(sleep_time: u64, side: BridgeSide, endpoints: Vec<String>) -> Self {
        Self {
            side,
            endpoints,
            sleep_time,
            ..Default::default()
        }
    }

    pub fn sleep_time(&self) -> u64 {
        self.sleep_time
    }

    fn get_first_endpoint(&self) -> Result<String, SentinelConfigError> {
        info!("getting first {} endpoint...", self.side());
        if self.endpoints.is_empty() {
            Err(SentinelConfigError::NoEndpoints(self.side))
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
            debug!("incrementing num rotations to {}", self.rotations);
        }
        warn!("on {} endpoint rotation #{}", self.side(), self.rotations);
        self.current = next;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    pub fn side(&self) -> BridgeSide {
        self.side
    }
}
