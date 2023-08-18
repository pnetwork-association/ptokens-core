use clap::{Parser, Subcommand};
use common_sentinel::LogLevel;

use crate::cmds::{
    CancelTxArgs,
    GetBalanceCliArgs,
    GetUserOpStateCliArgs,
    InitArgs,
    NonceCliArgs,
    ProcessBlockCliArgs,
    RemoveUserOpCliArgs,
    ResetCliArgs,
    SetGasPriceCliArgs,
    SubMatGetterArgs,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Log level - if extant, this overrides log level set in config
    #[arg(long, short)]
    log_level: Option<LogLevel>,

    #[command(subcommand)]
    pub cmds: Cmds,
}

impl Cli {
    pub fn log_level(&self) -> Option<log::Level> {
        self.log_level.map(|l| l.into())
    }
}

#[derive(Debug, Subcommand)]
pub enum Cmds {
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

    /// Reset the chain
    ResetChain(ResetCliArgs),

    /// Process block
    ProcessBlock(ProcessBlockCliArgs),

    /// Get user ops
    GetUserOps,

    /// Get user ops list
    GetUserOpList,

    /// Set gas price
    SetGasPrice(SetGasPriceCliArgs),

    /// Remove a user operation from the db
    RemoveUserOp(RemoveUserOpCliArgs),

    /// Cancel a user operation. Only works with ops already in the core
    CancelTx(CancelTxArgs),

    /// Get the user operation state. Only works with ops already in the core
    GetUserOpState(GetUserOpStateCliArgs),

    /// Get list of cancellable user ops
    GetCancellableOps,

    /// Generate an ETH private key
    GeneratePrivateKey,

    /// Get ETH balance of passed in address
    GetBalance(GetBalanceCliArgs),
}
