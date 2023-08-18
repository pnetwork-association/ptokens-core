use std::str::FromStr;

use clap::Args;
use common::BridgeSide;
use common_eth::convert_hex_to_eth_address;
use ethereum_types::Address as EthAddress;
use common_sentinel::{get_eth_balance, SentinelConfig, SentinelError};
use serde_json::json;

#[derive(Debug, Args)]
pub struct GetBalanceCliArgs {
    /// Native or host
    pub side: String,

    /// The address to get the balance of
    pub address: String,
}

#[derive(Clone, Debug)]
struct GetBalanceArgs {
    side: BridgeSide,
    address: EthAddress,
}

impl GetBalanceArgs {
    fn from_cli_args(GetBalanceCliArgs { side, address }: &GetBalanceCliArgs) -> Result<Self, SentinelError> {
        let side = BridgeSide::from_str(side)?;
        let address = convert_hex_to_eth_address(address)?;
        let r = Self { side, address };
        debug!("get balance cli args: {r:?}");
        Ok(r)
    }
}

pub async fn get_balance(config: &SentinelConfig, cli_args: &GetBalanceCliArgs) -> Result<String, SentinelError> {
    let GetBalanceArgs { address, side } = GetBalanceArgs::from_cli_args(cli_args)?;
    info!("getting {side} balance for address {address}...");
    let endpoints = if side.is_native() {
        config.get_native_endpoints()
    } else {
        config.get_host_endpoints()
    };
    let sleep_time = endpoints.sleep_time();
    let ws_client = endpoints.get_first_ws_client().await?;
    let balance = get_eth_balance(&ws_client, &address, sleep_time, side).await?;
    Ok(json!({ "jsonrpc": "2.0", "result": { "balance": balance, "address": address, "side": side}}).to_string())
}
