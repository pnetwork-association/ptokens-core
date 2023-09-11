use common::BridgeSide;
use common_eth::EthSubmissionMaterial;
use common_metadata::MetadataChainId;
use common_sentinel::{SentinelError, WebSocketMessagesEncodable, WebSocketMessagesError, WebSocketMessagesInitArgs};

use crate::android::State;

pub fn init<'a>(args: WebSocketMessagesInitArgs, state: State<'a>) -> Result<State<'a>, SentinelError> {
    if args.native_block().is_none() {
        let r = WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoBlock {
            side: BridgeSide::Native,
            struct_name: "init_args".to_string(),
        });
        Ok(state.add_response(r))
    } else if args.host_block().is_none() {
        let r = WebSocketMessagesEncodable::Error(WebSocketMessagesError::NoBlock {
            side: BridgeSide::Native,
            struct_name: "init_args".to_string(),
        });
        Ok(state.add_response(r))
    } else {
        let r = WebSocketMessagesEncodable::Success("we got here at least".into());
        Ok(state.add_response(r))
    }
}

/*
async fn init_native<D: DatabaseInterface>(
    db: &D,
    config: &SentinelConfig,
    args: &InitArgs,
) -> Result<(), SentinelError> {
    info!("Initializing native core...");
    let endpoints = config.get_native_endpoints();
    let ws_client = endpoints.get_first_ws_client().await?;
    let sleep_time = endpoints.sleep_time();
    let side = BridgeSide::Native;
    let latest_block_num = get_latest_block_num(&ws_client, sleep_time, side).await?;
    let sub_mat = get_sub_mat(&ws_client, latest_block_num, sleep_time, side).await?;

    init_v3_native_core(
        db,
        sub_mat,
        &config.native().get_eth_chain_id(),
        ZERO_GAS_PRICE,
        args.native_confs,
        &EthAddress::zero(), // NOTE: Vaults are not used in v3 sentinels
        &VaultUsingCores::from_core_type(&config.core().core_type)?,
        config.native().is_validating(),
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
    let ws_client = endpoints.get_first_ws_client().await?;
    let sleep_time = endpoints.sleep_time();
    let side = BridgeSide::Host;
    let latest_block_num = get_latest_block_num(&ws_client, sleep_time, side).await?;
    let sub_mat = get_sub_mat(&ws_client, latest_block_num, sleep_time, side).await?;

    init_v3_host_core(
        db,
        sub_mat,
        &config.host().get_eth_chain_id(),
        ZERO_GAS_PRICE,
        args.host_confs,
        config.host().is_validating(),
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
*/
