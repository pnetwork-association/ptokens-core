use common::{BridgeSide, DatabaseInterface};
use common_eth::{EthDbUtils, EthDbUtilsExt, EvmDbUtils};
use common_sentinel::{SentinelConfig, SentinelError, Side};
use serde_json::json;

#[derive(Debug, Args)]
pub struct SetGasPriceCliArgs {
    /// Which side of the bridge to set gas price of
    #[arg(value_enum)]
    pub side: Side,

    /// Gas price (unit: wei)
    pub price: u64,
}

pub async fn set_gas_price(config: &SentinelConfig, cli_args: &SetGasPriceCliArgs) -> Result<String, SentinelError> {
    let db = common_rocksdb_database::get_db_at_path(&config.get_db_path())?;
    let n_db_utils = EthDbUtils::new(&db);
    let h_db_utils = EvmDbUtils::new(&db);
    let s = &cli_args.side.into();

    db.start_transaction()?;
    if BridgeSide::is_native(s) {
        n_db_utils.put_eth_gas_price_in_db(cli_args.price)?
    } else {
        h_db_utils.put_eth_gas_price_in_db(cli_args.price)?
    };
    db.end_transaction()?;

    let r = json!({
        "jsonrpc": "2.0",
        "result": {
            "side": s,
            "gas_price": cli_args.price,
            "gas_price_set_success": true,
        }
    })
    .to_string();

    Ok(r)
}
