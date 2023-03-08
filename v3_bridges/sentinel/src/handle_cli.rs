use anyhow::Result;
use clap::Parser;
use lib::{init_logger, SentinelConfig};

use crate::{
    cli::{
        get_latest_block_num::{get_host_latest_block_num, get_native_latest_block_num},
        get_sub_mat::{get_host_sub_mat, get_native_sub_mat},
        CliArgs,
        SubCommands,
    },
    sentinel::start_sentinel,
};

pub async fn handle_cli() -> Result<String> {
    let config = SentinelConfig::new()?;
    init_logger(&config.log_config)?;
    let cli_args = CliArgs::parse();

    match cli_args.sub_commands {
        SubCommands::Start => start_sentinel(&config).await,
        SubCommands::GetHostSubMat(ref args) => get_host_sub_mat(&config.host_config.get_endpoints(), args).await,
        SubCommands::GetNativeSubMat(ref args) => get_native_sub_mat(&config.native_config.get_endpoints(), args).await,
        SubCommands::GetHostLatestBlockNum => get_host_latest_block_num(&config.host_config.get_endpoints()).await,
        SubCommands::GetNativeLatestBlockNum => {
            get_native_latest_block_num(&config.native_config.get_endpoints()).await
        },
    }
}
