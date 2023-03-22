use clap::Parser;

use crate::cli::{get_sub_mat::SubMatGetterArgs, init::InitArgs, NonceCliArgs, StartSentinelArgs};

#[derive(Debug, Parser)]
pub struct CliArgs {
    #[command(subcommand)]
    pub sub_commands: SubCommands,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    /// Start the Sentinel
    Start(StartSentinelArgs),

    /// Get HOST latest block number.
    GetHostLatestBlockNum,

    /// Get NATIVE latest block number.
    GetNativeLatestBlockNum,

    /// Get HOST submission material.
    GetHostSubMat(SubMatGetterArgs),

    /// Get NATIVE submission material.
    GetNativeSubMat(SubMatGetterArgs),

    /// Initialize the core.
    Init(InitArgs),

    /// Get the state of the core.
    GetCoreState,

    /// Get nonce for given address
    GetNonce(NonceCliArgs),
}
