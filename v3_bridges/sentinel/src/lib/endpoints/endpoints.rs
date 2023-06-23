use std::result::Result;

use common::BridgeSide;
use jsonrpsee::ws_client::WsClient;

use super::EndpointError;
use crate::{config::ConfigError, get_rpc_client, SentinelError};

const MAX_NUM_ROTATIONS: usize = 10; // TODO Make this configurable?

#[derive(Debug, Default, Clone)]
pub struct Endpoints {
    is_native: bool,
    sleep_time: u64,
    side: BridgeSide,
    endpoints: Vec<String>,
    current: usize,
    rotations: usize,
}

// TODO this file should return endpoint errors, not sentinel ones!

impl Endpoints {
    pub fn new(is_native: bool, sleep_time: u64, side: BridgeSide, endpoints: Vec<String>) -> Self {
        Self {
            side,
            is_native,
            endpoints,
            sleep_time,
            ..Default::default()
        }
    }

    fn get_first_endpoint(&self) -> Result<String, SentinelError> {
        info!("getting first {} endpoint...", self.side());
        if self.endpoints.is_empty() {
            Err(ConfigError::NoEndpoints(self.side).into())
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
        Ok(get_rpc_client(&self.endpoints[self.current]).await?)
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
        if self.rotations >= MAX_NUM_ROTATIONS {
            Err(EndpointError::MaxRotations(self.side(), MAX_NUM_ROTATIONS))
        } else {
            self.current = next;
            Ok(())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    fn side(&self) -> BridgeSide {
        self.side
    }
}
