use std::result::Result;

use clap::Parser;
use lib::{init_logger, SentinelConfig, SentinelError};
use serde_json::json;

use crate::{
    cli::{
        get_cancel_tx,
        get_cancellable_user_ops,
        get_core_state,
        get_host_latest_block_num,
        get_host_sub_mat,
        get_native_latest_block_num,
        get_native_sub_mat,
        get_nonce_cli,
        get_user_op_list,
        get_user_op_state,
        get_user_ops,
        init,
        process_block,
        remove_user_op,
        reset_chain_cli,
        set_gas_price,
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

    let r = match cli_args.sub_commands {
        SubCommands::GetUserOps => get_user_ops(&config),
        SubCommands::GetCoreState => get_core_state(&config),
        SubCommands::GetUserOpList => get_user_op_list(&config),
        SubCommands::Init(ref args) => init(&config, args).await,
        SubCommands::Start(ref args) => start_sentinel(&config, args).await,
        SubCommands::GetNonce(ref args) => get_nonce_cli(&config, args).await,
        SubCommands::CancelTx(ref args) => get_cancel_tx(&config, args).await,
        SubCommands::SetGasPrice(ref args) => set_gas_price(&config, args).await,
        SubCommands::GetCancellableOps => get_cancellable_user_ops(&config).await,
        SubCommands::ResetChain(ref args) => reset_chain_cli(&config, args).await,
        SubCommands::ProcessBlock(ref args) => process_block(&config, args).await,
        SubCommands::RemoveUserOp(ref args) => remove_user_op(&config, args).await,
        SubCommands::GetUserOpState(ref args) => get_user_op_state(&config, args).await,
        SubCommands::GetHostSubMat(ref args) => {
            get_host_sub_mat(&config.host().endpoints().get_first_ws_client().await?, args).await
        },
        SubCommands::GetHostLatestBlockNum => {
            get_host_latest_block_num(&config.host().endpoints().get_first_ws_client().await?).await
        },
        SubCommands::GetNativeSubMat(ref args) => {
            get_native_sub_mat(&config.native().endpoints().get_first_ws_client().await?, args).await
        },
        SubCommands::GetNativeLatestBlockNum => {
            get_native_latest_block_num(&config.native().endpoints().get_first_ws_client().await?).await
        },
    };

    r.map_err(|e| SentinelError::Json(json!({"jsonrpc": "2.0", "error": e.to_string()})))
}
