use clap::Parser;

use crate::cli::get_sub_mat::SubMatGetterArgs;

#[derive(Debug, Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub sub_commands: SubCommands,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    /// Get HOST latest block number.
    GetHostLatestBlockNum,

    /// Get NATIVE latest block number.
    GetNativeLatestBlockNum,

    /// Get HOST submission material.
    GetHostSubMat(SubMatGetterArgs),

    /// Get NATIVE submission material.
    GetNativeSubMat(SubMatGetterArgs),
}
