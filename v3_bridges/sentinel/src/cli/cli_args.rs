use clap::{Args, Parser, Subcommand};

use crate::cli::get_submission_material::GetSubmissionMaterialSubCommand;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliArgs {
    /// Get submission material.
    #[clap(subcommand)]
    pub get_submission_material: GetSubmissionMaterialSubCommand,
}
