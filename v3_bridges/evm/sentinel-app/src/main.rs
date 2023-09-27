mod broadcaster;
mod cli;
mod eth_rpc;
mod rpc_server;
mod start_sentinel;
mod syncer;
mod ws_server;

#[macro_use]
extern crate log;

use std::result::Result;

use clap::Parser;
use cli::handle_cli;
use common_sentinel::{init_logger, LogLevel, SentinelConfig, SentinelError};
use serde_json::json;

use self::cli::Commands;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(rename_all = "camelCase")]
pub struct Cli {
    /// Log level - if extant, this overrides log level set in config
    #[arg(long, short)]
    log_level: Option<LogLevel>,

    /// Configuration toml file path (default `./sentinel-config`)
    #[arg(long, short)]
    config_path: Option<String>,

    /// Disable the ws server
    #[arg(short = 'y', long)]
    disable_ws_server: bool,

    /// Disable the rpc server
    #[arg(short = 'z', long)]
    disable_rpc_server: bool,

    // NOTE: These are optional, if no command is passed the sentinel is started proper
    #[command(subcommand)]
    commands: Option<Commands>,
}

impl Cli {
    pub fn log_level(&self) -> Option<log::Level> {
        self.log_level.map(|l| l.into())
    }
}

async fn start() -> Result<String, SentinelError> {
    let cli_args = Cli::parse();
    let config_path = if let Some(ref path) = cli_args.config_path {
        path.to_string()
    } else {
        "sentinel-config".to_string()
    };
    let config = SentinelConfig::new(&config_path)?;

    if config.log().is_enabled() {
        init_logger(config.log(), cli_args.log_level())?
    };

    if let Some(commands) = cli_args.commands {
        handle_cli(commands).await
    } else {
        let r = start_sentinel::start_sentinel(&config, cli_args.disable_rpc_server, cli_args.disable_ws_server).await;
        r.map_err(|e| SentinelError::Json(json!({"jsonrpc": "2.0", "error": e.to_string()})))
    }
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
