use common::{CoreType, DatabaseInterface};
use common_eth::{convert_hex_to_eth_address, init_v3_host_core, init_v3_native_core, VaultUsingCores};
use common_rocksdb::get_db_at_path;
use lib::{get_latest_block_num, get_sub_mat, ConfigT, SentinelConfig, SentinelError};
use serde_json::json;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// The host side gas price to use when creating transactions (Unit: wei).
    pub host_gas_price: u64,

    /// The native side gas price to use when creating transactions (Unit: wei).
    pub native_gas_price: u64,

    /// Number of native side confirmations before creating transactions.
    pub native_confs: u64,

    /// Number of host side confirmations before creating transactions.
    pub host_confs: u64,

    /// The address of the vault contract.
    pub vault_address: String,
    // FIXME Rm!
}

async fn init_native<D: DatabaseInterface>(
    db: &D,
    config: &SentinelConfig,
    args: &InitArgs,
) -> Result<(), SentinelError> {
    info!("Initializing native core...");
    let endpoints = config.get_native_endpoints();
    let ws_client = endpoints.get_rpc_client().await?;
    let latest_block_num = get_latest_block_num(&endpoints).await?;
    let sub_mat = get_sub_mat(&ws_client, latest_block_num).await?;

    init_v3_native_core(
        db,
        sub_mat,
        &config.native_config.get_eth_chain_id(),
        args.native_gas_price,
        args.native_confs,
        &convert_hex_to_eth_address(&args.vault_address)?,
        &VaultUsingCores::from_core_type(&config.core_config.core_type)?,
        config.native_config.is_validating(),
    )?;

    Ok(())
}

async fn init_host<D: DatabaseInterface>(
    db: &D,
    config: &SentinelConfig,
    args: &InitArgs,
) -> Result<(), SentinelError> {
    info!("Initializing host core...");
    let endpoints = config.get_host_endpoints();
    let ws_client = endpoints.get_rpc_client().await?;
    let latest_block_num = get_latest_block_num(&endpoints).await?;
    let sub_mat = get_sub_mat(&ws_client, latest_block_num).await?;

    init_v3_host_core(
        db,
        sub_mat,
        &config.host_config.get_eth_chain_id(),
        args.host_gas_price,
        args.host_confs,
        config.host_config.is_validating(),
    )?;

    Ok(())
}

pub async fn init(config: &SentinelConfig, args: &InitArgs) -> Result<String, SentinelError> {
    info!("Initializing core...");
    let db = get_db_at_path(&config.get_db_path())?;
    db.start_transaction()?;

    let host_is_initted = CoreType::host_core_is_initialized(&db);
    let native_is_initted = CoreType::native_core_is_initialized(&db);

    if !native_is_initted {
        init_native(&db, config, args).await?;
    }

    if !host_is_initted {
        init_host(&db, config, args).await?;
    }

    if native_is_initted && host_is_initted {
        warn!("Core already initialized!")
    }

    db.end_transaction()?;

    Ok(json!({
        "jsonrpc": "2.0",
        "result": "core initialized",
    })
    .to_string())
}
