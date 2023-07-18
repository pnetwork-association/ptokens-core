use std::{fs::read_to_string, path::Path, str::FromStr};

use clap::Args;
use common::{BridgeSide, DatabaseInterface};
use common_eth::{EthDbUtils, EthDbUtilsExt, EthState, EthSubmissionMaterial, EvmDbUtils};
use common_eth_debug::reset_eth_chain;
use lib::{get_latest_block_num, get_sub_mat, SentinelConfig, SentinelError, Side};
use serde_json::json;

#[derive(Debug, Args)]
pub struct ResetCliArgs {
    /// Which side of the bridge to reset
    #[arg(value_enum)]
    side: Side,

    /// If no arguement provided, it will reset to the latest block
    #[command(flatten)]
    arg_group: ArgGroup,

    /// Optional number of confirmations. If omitted it will use the previous value instead
    #[arg(long, short)]
    confs: Option<u64>,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct ArgGroup {
    /// Path to block to reset to
    #[arg(long, short)]
    path: Option<String>,

    /// Block number to reset to
    #[arg(long, short)]
    block_num: Option<u64>,

    /// Use the latest block to reset to
    #[arg(long, short)]
    latest: Option<bool>,
}

#[derive(Clone, Debug)]
struct ResetArgs {
    side: BridgeSide,
    confs: Option<u64>,
    block_num: Option<u64>,
    block: Option<EthSubmissionMaterial>,
}

impl ResetArgs {
    fn from_cli_args(cli_args: &ResetCliArgs) -> Result<Self, SentinelError> {
        let side = cli_args.side.into();
        let block = if let Some(ref p) = cli_args.arg_group.path {
            Some(Self::get_block_from_path(p)?)
        } else {
            None
        };
        let r = Self {
            side,
            block,
            confs: cli_args.confs,
            block_num: cli_args.arg_group.block_num,
        };
        debug!("reset cli args: {r:?}");
        Ok(r)
    }

    fn get_block_from_path(s: &str) -> Result<EthSubmissionMaterial, SentinelError> {
        let p = Path::new(s);
        if p.exists() {
            Ok(EthSubmissionMaterial::from_str(&read_to_string(p)?)?)
        } else {
            Err(SentinelError::Custom(format!("Block not found @ '{}'", p.display())))
        }
    }
}

pub async fn reset_chain_cli(config: &SentinelConfig, cli_args: &ResetCliArgs) -> Result<String, SentinelError> {
    info!("resetting chain...");
    if !config.core().db_exists() {
        return Err(SentinelError::Custom(format!(
            "cannot find db @ path: '{}'",
            config.core().db_path
        )));
    };
    let db = common_rocksdb_database::get_db_at_path(&config.get_db_path())?;
    let args = ResetArgs::from_cli_args(cli_args)?;
    let side = args.side;
    let native_db_utils = EthDbUtils::new(&db);
    let host_db_utils = EvmDbUtils::new(&db);

    let ws_client = if args.side.is_native() {
        config.native().endpoints().get_first_ws_client().await?
    } else {
        config.host().endpoints().get_first_ws_client().await?
    };
    let sleep_time = if args.side.is_native() {
        config.native().endpoints().sleep_time()
    } else {
        config.host().endpoints().sleep_time()
    };

    let sub_mat = if let Some(b) = args.block {
        debug!("resetting {side} using block from path");
        b
    } else if let Some(n) = args.block_num {
        debug!("resetting {side} with supplied block num {n}");
        get_sub_mat(&ws_client, n, sleep_time, args.side).await?
    } else {
        let l = get_latest_block_num(&ws_client, sleep_time, args.side).await?;
        debug!("resetting {side} with latest block num {l}");
        get_sub_mat(&ws_client, l, sleep_time, args.side).await?
    };

    let confs = match args.confs {
        Some(c) => c,
        None => {
            if args.side.is_native() {
                native_db_utils.get_eth_canon_to_tip_length_from_db()?
            } else {
                host_db_utils.get_eth_canon_to_tip_length_from_db()?
            }
        },
    };

    let reset_block_num = sub_mat.get_block_number()?.as_u64();

    db.start_transaction()?;
    reset_eth_chain(
        EthState::init(&db).add_eth_submission_material(sub_mat)?,
        confs,
        args.side.is_native(),
    )?;
    db.end_transaction()?;

    let r = json!({
        "jsonrpc": "2.0",
        "result": { "side": side, "chain_reset_success": true, "reset_block_num": reset_block_num },
    })
    .to_string();

    Ok(r)
}
