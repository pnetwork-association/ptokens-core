use std::result::Result;

use lib::{SentinelConfig, SentinelError};

use super::{
    generate_private_key,
    get_balance,
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
    CliSubCommands,
};

pub async fn handle_cli(config: &SentinelConfig, cmds: &CliSubCommands) -> Result<String, SentinelError> {
    match cmds {
        CliSubCommands::GetUserOps => get_user_ops(config),
        CliSubCommands::GetCoreState => get_core_state(config),
        CliSubCommands::GetUserOpList => get_user_op_list(config),
        CliSubCommands::Init(ref args) => init(config, args).await,
        CliSubCommands::GeneratePrivateKey => generate_private_key(),
        CliSubCommands::GetNonce(ref args) => get_nonce_cli(config, args).await,
        CliSubCommands::CancelTx(ref args) => get_cancel_tx(config, args).await,
        CliSubCommands::GetBalance(ref args) => get_balance(config, args).await,
        CliSubCommands::SetGasPrice(ref args) => set_gas_price(config, args).await,
        CliSubCommands::GetCancellableOps => get_cancellable_user_ops(config).await,
        CliSubCommands::ResetChain(ref args) => reset_chain_cli(config, args).await,
        CliSubCommands::ProcessBlock(ref args) => process_block(config, args).await,
        CliSubCommands::RemoveUserOp(ref args) => remove_user_op(config, args).await,
        CliSubCommands::GetUserOpState(ref args) => get_user_op_state(config, args).await,
        CliSubCommands::GetHostSubMat(ref args) => {
            get_host_sub_mat(&config.host().endpoints().get_first_ws_client().await?, args).await
        },
        CliSubCommands::GetHostLatestBlockNum => {
            get_host_latest_block_num(&config.host().endpoints().get_first_ws_client().await?).await
        },
        CliSubCommands::GetNativeSubMat(ref args) => {
            get_native_sub_mat(&config.native().endpoints().get_first_ws_client().await?, args).await
        },
        CliSubCommands::GetNativeLatestBlockNum => {
            get_native_latest_block_num(&config.native().endpoints().get_first_ws_client().await?).await
        },
    }
}
