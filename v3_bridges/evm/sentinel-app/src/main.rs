mod broadcaster;
mod core;
mod eth_rpc;
mod rpc_server;
mod start_sentinel;
mod syncer;
mod ws_server;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use std::result::Result;

use clap::Parser;
use common_sentinel::{init_logger, LogLevel, SentinelConfig, SentinelError};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Log level - if extant, this overrides log level set in config
    #[arg(long, short)]
    log_level: Option<LogLevel>,

    /// Disable the ws server
    #[arg(short = 'v', long)]
    disable_ws_server: bool,

    /// Disable the native syncer
    #[arg(short = 'w', long)]
    disable_native_syncer: bool,

    /// Disable the host syncer
    #[arg(short = 'x', long)]
    disable_host_syncer: bool,

    /// Disable the broadcaster
    #[arg(short = 'y', long)]
    disable_broadcaster: bool,

    /// Disable the rpc server
    #[arg(short = 'z', long)]
    disable_rpc_server: bool,
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

    let r = start_sentinel::start_sentinel(
        &config,
        cli_args.disable_native_syncer,
        cli_args.disable_host_syncer,
        cli_args.disable_broadcaster,
        cli_args.disable_rpc_server,
        cli_args.disable_ws_server,
    )
    .await;

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
