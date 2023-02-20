use clap::Args;

#[derive(Debug, Subcommand)]
pub enum GetSubmissionMaterialSubCommand {
    /// Get HOST submission material.
    GetHostSubmissionMaterial(GetSubmissionMaterialCommand),

    /// Get NATIVE submission material.
    GetNativeSubmissionMaterial(GetSubmissionMaterialCommand),
}

#[derive(Debug, Args)]
pub struct GetSubmissionMaterialCommand {
    /// Block number to create the submission material for.
    pub block_num: u64,

    /// Optional path to save the submission material to.
    #[arg(long, short)]
    pub path: Option<String>,
}
