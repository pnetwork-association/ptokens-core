use std::{convert::TryFrom, fs::read_to_string, path::Path, str::FromStr};

use clap::Args;
use common::BridgeSide;
use common_eth::EthSubmissionMaterial;
use common_rocksdb::get_db_at_path;
use derive_more::Constructor;
use lib::{SentinelConfig, SentinelError};
use serde_json::json;

use crate::sentinel::{process_host, process_native};

#[derive(Clone, Debug, Default, Args)]
pub struct ProcessBlockCliArgs {
    /// Which side of the bridge to reset
    pub side: String,

    /// Path to block.
    pub path: String,

    /// Dry run (nothing is commited to the databases)
    #[arg(long, short)]
    pub dry_run: Option<bool>,
}

#[derive(Clone, Debug, Default, Constructor)]
pub struct ProcessBlockArgs {
    pub dry_run: bool,
    pub side: BridgeSide,
    pub sub_mat: EthSubmissionMaterial,
}

impl TryFrom<&ProcessBlockCliArgs> for ProcessBlockArgs {
    type Error = SentinelError;

    fn try_from(a: &ProcessBlockCliArgs) -> Result<Self, Self::Error> {
        let side = BridgeSide::from_str(&a.side)?;
        let p = Path::new(&a.path);
        if !p.exists() {
            return Err(SentinelError::Custom(format!("Cannot find block @ path: `{}`", a.path)));
        }
        let sub_mat = EthSubmissionMaterial::from_str(&read_to_string(p)?)?;
        let dry_run = match a.dry_run {
            Some(true) => true,
            _ => {
                warn!("Dry run set to false - changes could be committed to the db!");
                false
            },
        };
        Ok(Self::new(dry_run, side, sub_mat))
    }
}

pub async fn process_block(config: &SentinelConfig, cli_args: &ProcessBlockCliArgs) -> Result<String, SentinelError> {
    let args = ProcessBlockArgs::try_from(cli_args)?;
    let db = get_db_at_path(&config.get_db_path())?;
    let is_in_sync = true;
    let state_manager = config.get_state_manager(&args.side);
    let is_validating = true;
    let use_db_tx = !args.dry_run;
    let output = if args.side.is_native() {
        process_native(
            &db,
            is_in_sync,
            &args.sub_mat,
            &state_manager,
            is_validating,
            use_db_tx,
            args.dry_run,
        )?
    } else {
        process_host(
            &db,
            is_in_sync,
            &args.sub_mat,
            &state_manager,
            is_validating,
            use_db_tx,
            args.dry_run,
        )?
    };
    let latest_block_num = args.sub_mat.get_block_number()?;
    let r = json!({
        "jsonrpc": "2.0",
        "result": {
            "user_ops": output,
            "used_db_tx": use_db_tx,
            "dry_run": args.dry_run,
            "is_validating": is_validating,
            "latest_block_num": latest_block_num,
        }
    });
    Ok(r.to_string())
}
