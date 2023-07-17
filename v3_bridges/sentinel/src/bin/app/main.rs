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
    cli::{handle_cli, CliArgs, CliSubCommands},
    sentinel::start_sentinel,
};

async fn start() -> Result<String, SentinelError> {
    let config = SentinelConfig::new()?;

    let cli_args = CliArgs::parse();

    if config.log().is_enabled() {
        init_logger(config.log(), cli_args.log_level())?
    };

    let r = match cli_args.sub_commands {
        CliSubCommands::Start(ref args) => start_sentinel(&config, args).await,
        _ => handle_cli(&config, &cli_args).await,
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
