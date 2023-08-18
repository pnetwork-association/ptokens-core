use std::result::Result;

use clap::Parser;
use common_sentinel::{init_logger, SentinelConfig, SentinelError};

use crate::{
    cli::{Cli, Cmds},
    cmds,
};

pub async fn handle_cli() -> Result<String, SentinelError> {
    let config = SentinelConfig::new()?;

    let cli = Cli::parse();

    if config.log().is_enabled() {
        init_logger(config.log(), cli.log_level())?
    };

    match cli.cmds {
        Cmds::GetUserOps => cmds::get_user_ops(&config),
        Cmds::GetCoreState => cmds::get_core_state(&config),
        Cmds::GetUserOpList => cmds::get_user_op_list(&config),
        Cmds::Init(ref args) => cmds::init(&config, args).await,
        Cmds::GeneratePrivateKey => cmds::generate_private_key(),
        Cmds::GetNonce(ref args) => cmds::get_nonce_cli(&config, args).await,
        Cmds::CancelTx(ref args) => cmds::get_cancel_tx(&config, args).await,
        Cmds::GetBalance(ref args) => cmds::get_balance(&config, args).await,
        Cmds::SetGasPrice(ref args) => cmds::set_gas_price(&config, args).await,
        Cmds::GetCancellableOps => cmds::get_cancellable_user_ops(&config).await,
        Cmds::ResetChain(ref args) => cmds::reset_chain_cli(&config, args).await,
        Cmds::ProcessBlock(ref args) => cmds::process_block(&config, args).await,
        Cmds::RemoveUserOp(ref args) => cmds::remove_user_op(&config, args).await,
        Cmds::GetUserOpState(ref args) => cmds::get_user_op_state(&config, args).await,
        Cmds::GetHostSubMat(ref args) => {
            cmds::get_host_sub_mat(&config.host().endpoints().get_first_ws_client().await?, args).await
        },
        Cmds::GetHostLatestBlockNum => {
            cmds::get_host_latest_block_num(&config.host().endpoints().get_first_ws_client().await?).await
        },
        Cmds::GetNativeSubMat(ref args) => {
            cmds::get_native_sub_mat(&config.native().endpoints().get_first_ws_client().await?, args).await
        },
        Cmds::GetNativeLatestBlockNum => {
            cmds::get_native_latest_block_num(&config.native().endpoints().get_first_ws_client().await?).await
        },
    }
}
