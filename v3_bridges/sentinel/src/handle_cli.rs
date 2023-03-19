use std::result::Result;

use clap::Parser;
use lib::{init_logger, SentinelConfig, SentinelError};

use crate::{
    cli::{
        get_core_state,
        get_host_latest_block_num,
        get_host_sub_mat,
        get_native_latest_block_num,
        get_native_sub_mat,
        init,
        CliArgs,
        SubCommands,
    },
    sentinel::start_sentinel,
};

pub async fn handle_cli() -> Result<String, SentinelError> {
    let config = SentinelConfig::new()?;

    if config.log_config.is_enabled() {
        init_logger(&config.log_config)?;
    };

    let cli_args = CliArgs::parse();

    match cli_args.sub_commands {
        SubCommands::Start(ref args) => start_sentinel(&config, args).await,
        SubCommands::GetCoreState => get_core_state(&config),
        SubCommands::Init(ref args) => init(&config, args).await,
        SubCommands::GetHostSubMat(ref args) => get_host_sub_mat(&config.host_config.get_endpoints(), args).await,
        SubCommands::GetNativeSubMat(ref args) => get_native_sub_mat(&config.native_config.get_endpoints(), args).await,
        SubCommands::GetHostLatestBlockNum => get_host_latest_block_num(&config.host_config.get_endpoints()).await,
        SubCommands::GetNativeLatestBlockNum => {
            get_native_latest_block_num(&config.native_config.get_endpoints()).await
        },
    }
}
