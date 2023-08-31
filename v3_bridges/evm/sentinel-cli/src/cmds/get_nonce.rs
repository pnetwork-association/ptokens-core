use std::str::FromStr;

use clap::Args;
use common::BridgeSide;
use common_eth::convert_hex_to_eth_address;
use common_sentinel::{get_nonce, SentinelConfig, SentinelError};
use ethereum_types::Address as EthAddress;
use serde_json::json;

#[derive(Debug, Args)]
pub struct NonceCliArgs {
    /// Native or host
    pub side: String,

    /// The address to get the nonce of
    pub address: String,
}

#[derive(Clone, Debug)]
struct NonceArgs {
    side: BridgeSide,
    address: EthAddress,
}

impl NonceArgs {
    fn from_cli_args(NonceCliArgs { side, address }: &NonceCliArgs) -> Result<Self, SentinelError> {
        let side = BridgeSide::from_str(side)?;
        let address = convert_hex_to_eth_address(address)?;
        let r = Self { side, address };
        debug!("Nonce cli args: {r:?}");
        Ok(r)
    }
}

pub async fn get_nonce_cli(config: &SentinelConfig, cli_args: &NonceCliArgs) -> Result<String, SentinelError> {
    let NonceArgs { address, side } = NonceArgs::from_cli_args(cli_args)?;
    info!("Getting {side} nonce for address {address}...");
    let endpoints = if side.is_native() {
        config.get_native_endpoints()
    } else {
        config.get_host_endpoints()
    };
    let sleep_time = endpoints.sleep_time();
    let ws_client = endpoints.get_first_ws_client().await?;
    let nonce = get_nonce(&ws_client, &address, sleep_time, side).await?;
    Ok(json!({ "jsonrpc": "2.0", "result": { "nonce": nonce, "address": address, "side": side}}).to_string())
}
