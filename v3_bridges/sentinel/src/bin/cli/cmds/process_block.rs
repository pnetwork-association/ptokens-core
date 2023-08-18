use std::{convert::TryFrom, fs::read_to_string, path::Path, str::FromStr};

use clap::Args;
use common::BridgeSide;
use common_eth::EthSubmissionMaterial;
use common_rocksdb_database::get_db_at_path;
use derive_more::Constructor;
use lib::{
    get_sub_mat,
    process_single,
    NetworkId,
    SentinelConfig,
    SentinelError,
    DEFAULT_SLEEP_TIME,
    HOST_PROTOCOL_ID,
    NATIVE_PROTOCOL_ID,
};
use serde_json::json;

#[derive(Clone, Debug, Default, Args)]
pub struct ProcessBlockCliArgs {
    /// Which side of the bridge to process a block for
    side: String,

    #[command(flatten)]
    arg_group: ArgGroup,

    /// Dry run (nothing is commited to the databases)
    #[arg(long, short)]
    dry_run: Option<bool>,

    /// Reprocess block. This skips appending the block to the chain.
    #[arg(long, short)]
    reprocess: Option<bool>,
}

#[derive(Default, Clone, Debug, Args)]
#[group(required = true, multiple = false)]
struct ArgGroup {
    /// Path to block to process
    #[arg(long, short)]
    path: Option<String>,

    /// Block number to process
    #[arg(long, short)]
    block_num: Option<u64>,
}

#[derive(Clone, Debug, Default, Constructor)]
pub struct ProcessBlockArgs {
    dry_run: bool,
    reprocess: bool,
    side: BridgeSide,
    block_num: Option<u64>,
    sub_mat: Option<EthSubmissionMaterial>,
}

impl TryFrom<&ProcessBlockCliArgs> for ProcessBlockArgs {
    type Error = SentinelError;

    fn try_from(a: &ProcessBlockCliArgs) -> Result<Self, Self::Error> {
        let side = BridgeSide::from_str(&a.side)?;

        let sub_mat = if let Some(ref path) = a.arg_group.path {
            let p = Path::new(path);

            if !p.exists() {
                // TODO Need some specific cli arg error handling for neatness
                return Err(SentinelError::Custom(format!(
                    "Cannot find block @ path: `{}`",
                    a.arg_group.path.as_ref().unwrap()
                )));
            }

            Some(EthSubmissionMaterial::from_str(&read_to_string(p)?)?)
        } else {
            None
        };

        let dry_run = matches!(a.dry_run, Some(true));
        let reprocess = matches!(a.reprocess, Some(true));

        if !dry_run {
            warn!("dry run is set to false - changes will be committed to the db!");
        };

        Ok(Self::new(dry_run, reprocess, side, a.arg_group.block_num, sub_mat))
    }
}

pub async fn process_block(config: &SentinelConfig, cli_args: &ProcessBlockCliArgs) -> Result<String, SentinelError> {
    let args = ProcessBlockArgs::try_from(cli_args)?;
    let db = get_db_at_path(&config.get_db_path())?;
    let side = args.side;
    let pnetwork_hub = config.pnetwork_hub(&side);
    let is_validating = config.is_validating(&side);
    let dry_run = args.dry_run;
    let use_db_tx = !dry_run;

    let network_id = &NetworkId::new(
        if side.is_native() {
            config.native().get_eth_chain_id()
        } else {
            config.host().get_eth_chain_id()
        },
        if side.is_native() {
            *NATIVE_PROTOCOL_ID
        } else {
            *HOST_PROTOCOL_ID
        },
    )
    .to_bytes_4()?;

    let sub_mat = if let Some(sub_mat) = args.sub_mat {
        sub_mat
    } else {
        let n = args.block_num.unwrap_or_default();
        let ws_client = if side.is_native() {
            config.native().endpoints().get_first_ws_client().await?
        } else {
            config.host().endpoints().get_first_ws_client().await?
        };

        get_sub_mat(&ws_client, n, DEFAULT_SLEEP_TIME, side).await?
    };

    let processed_block_num = sub_mat.get_block_number()?;

    let processed_user_ops = process_single(
        &db,
        &sub_mat,
        &pnetwork_hub,
        is_validating,
        use_db_tx,
        dry_run,
        args.side,
        network_id,
        args.reprocess,
    )?;

    let r = json!({
        "jsonrpc": "2.0",
        "result": {
            "dry_run": dry_run,
            "use_db_tx": use_db_tx,
            "is_validating": is_validating,
            "processed_user_ops": processed_user_ops,
            "processed_block_num": processed_block_num,
        }
    })
    .to_string();

    Ok(r)
}