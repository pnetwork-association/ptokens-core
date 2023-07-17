mod cli;
mod sentinel;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use std::result::Result;

use clap::Parser;
use lib::{init_logger, SentinelConfig, SentinelError};
use serde_json::json;

use crate::{
    cli::{handle_cli, CliSubCommands, LogLevel},
    sentinel::start_sentinel,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Log level - if extant, this overrides log level set in config
    #[arg(long, short)]
    log_level: Option<LogLevel>,

    /// Disable the native syncer
    #[arg(short = 'x', long)]
    disable_native_syncer: bool,

    /// Disable the host syncer
    #[arg(short = 'y', long)]
    disable_host_syncer: bool,

    /// Disable the broadcaster
    #[arg(short = 'z', long)]
    disable_broadcaster: bool,

    #[command(subcommand)]
    sub_commands: Option<CliSubCommands>,
}

impl Cli {
    pub fn log_level(&self) -> Option<log::Level> {
        self.log_level.map(|l| l.into())
    }
}

async fn start() -> Result<String, SentinelError> {
    let config = SentinelConfig::new()?;

    let cli_args = Cli::parse();

    if config.log().is_enabled() {
        init_logger(config.log(), cli_args.log_level())?
    };

    let r = if let Some(cmds) = cli_args.sub_commands {
        handle_cli(&config, &cmds).await
    } else {
        start_sentinel(
            &config,
            cli_args.disable_native_syncer,
            cli_args.disable_host_syncer,
            cli_args.disable_broadcaster,
        )
        .await
    };

    r.map_err(|e| SentinelError::Json(json!({"jsonrpc": "2.0", "error": e.to_string()})))
}

#[tokio::main]
async fn main() {
    match start().await {
        Ok(s) => {
            info!("{s}");
            println!("{s}");
        },
        Err(e) => {
            error!("{e}");
            eprintln!("{e}");
            std::process::exit(1)
        },
    };
}
