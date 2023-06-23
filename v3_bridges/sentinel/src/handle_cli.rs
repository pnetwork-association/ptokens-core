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
        get_nonce_cli,
        get_user_ops,
        init,
        process_block,
        reset_chain_cli,
        CliArgs,
        SubCommands,
    },
    sentinel::start_sentinel,
};

pub async fn handle_cli() -> Result<String, SentinelError> {
    let config = SentinelConfig::new()?;

    if config.log().is_enabled() {
        init_logger(config.log())?;
    };

    let cli_args = CliArgs::parse();

    let h_ws_client = config.host().endpoints().get_first_ws_client().await?;
    let n_ws_client = config.native().endpoints().get_first_ws_client().await?;

    match cli_args.sub_commands {
        SubCommands::GetUserOps => get_user_ops(&config),
        SubCommands::GetCoreState => get_core_state(&config),
        SubCommands::Init(ref args) => init(&config, args).await,
        SubCommands::Start(ref args) => start_sentinel(&config, args).await,
        SubCommands::GetNonce(ref args) => get_nonce_cli(&config, args).await,
        SubCommands::ResetChain(ref args) => reset_chain_cli(&config, args).await,
        SubCommands::ProcessBlock(ref args) => process_block(&config, args).await,
        SubCommands::GetHostSubMat(ref args) => get_host_sub_mat(&h_ws_client, args).await,
        SubCommands::GetHostLatestBlockNum => get_host_latest_block_num(&h_ws_client).await,
        SubCommands::GetNativeSubMat(ref args) => get_native_sub_mat(&n_ws_client, args).await,
        SubCommands::GetNativeLatestBlockNum => get_native_latest_block_num(&n_ws_client).await,
    }
}
