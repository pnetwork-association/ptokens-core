use clap::{Parser};

use crate::cli::get_sub_mat::GetSubMatSubCommand;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliArgs {
    /// Get submission material.
    #[clap(subcommand)]
    pub get_sub_mat: GetSubMatSubCommand,
}
