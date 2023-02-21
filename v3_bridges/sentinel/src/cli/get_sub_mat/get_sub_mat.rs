use clap::Args;

#[derive(Debug, Subcommand)]
pub enum GetSubMatSubCommand {
    /// Get HOST submission material.
    GetHostSubMat(GetSubMatCommand),

    /// Get NATIVE submission material.
    GetNativeSubMat(GetSubMatCommand),
}

#[derive(Debug, Args)]
pub struct GetSubMatCommand {
    /// Block number to create the submission material for.
    pub block_num: u64,

    /// Optional path to save the submission material to.
    #[arg(long, short)]
    pub path: Option<String>,
}
