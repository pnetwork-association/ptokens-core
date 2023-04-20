use std::{fs::read_to_string, path::Path, str::FromStr};

use clap::Args;
use common::{BridgeSide, DatabaseInterface};
use common_eth::{EthDbUtils, EthDbUtilsExt, EthState, EthSubmissionMaterial, EvmDbUtils};
use common_eth_debug::reset_eth_chain;
use lib::{get_latest_block_num, get_sub_mat, SentinelConfig, SentinelError};
use serde_json::json;

#[derive(Debug, Args)]
pub struct ResetCliArgs {
    /// Which side of the bridge to reset
    pub side: String,

    /// Optional path to block. If omitted it will reset using the latest block instead.
    #[arg(long, short)]
    pub path: Option<String>,

    /// Optional number of confirmations. If omitted it will use the previous value instead.
    #[arg(long, short)]
    pub confs: Option<u64>,
}

#[derive(Clone, Debug)]
struct ResetArgs {
    side: BridgeSide,
    confs: Option<u64>,
    block: Option<EthSubmissionMaterial>,
}

impl ResetArgs {
    fn from_cli_args(cli_args: &ResetCliArgs) -> Result<Self, SentinelError> {
        let side = BridgeSide::from_str(&cli_args.side)?;
        let block = if let Some(ref p) = cli_args.path {
            Some(Self::get_block_from_path(p)?)
        } else {
            None
        };
        let r = Self {
            side,
            block,
            confs: cli_args.confs,
        };
        debug!("Reset cli args: {r:?}");
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
    info!("Resetting chain...");
    if !config.core().db_exists() {
        return Err(SentinelError::Custom(format!(
            "Cannot find db @ path: '{}'",
            config.core().db_path
        )));
    };
    let db = common_rocksdb::get_db_at_path(&config.get_db_path())?;
    let args = ResetArgs::from_cli_args(cli_args)?;
    let native_db_utils = EthDbUtils::new(&db);
    let host_db_utils = EvmDbUtils::new(&db);

    let endpoints = if args.side.is_native() {
        config.native().endpoints()
    } else {
        config.host().endpoints()
    };
    let sub_mat = match args.block {
        Some(b) => b,
        None => {
            let n = get_latest_block_num(&endpoints).await?;
            get_sub_mat(&endpoints, n).await?
        },
    };
    let block_num = sub_mat.get_block_number()?;
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

    db.start_transaction()?;
    reset_eth_chain(
        EthState::init(&db).add_eth_submission_material(sub_mat)?,
        confs,
        args.side.is_native(),
    )?;
    db.end_transaction()?;

    Ok(json!({
        "jsonrpc": "2.0",
        "result": {
            "side": args.side,
            "chain_reset_success": true,
            "latest_block_num": block_num.as_u64(),
        }
    })
    .to_string())
}
