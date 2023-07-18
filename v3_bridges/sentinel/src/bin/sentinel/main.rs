mod broadcaster;
mod core;
mod eth_rpc;
mod mongo;
mod processor;
mod rpc_server;
mod start_sentinel;
mod syncer;

use self::{
    broadcaster::broadcaster_loop,
    core::core_loop,
    eth_rpc::eth_rpc_loop,
    mongo::mongo_loop,
    processor::processor_loop,
    rpc_server::rpc_server_loop,
    start_sentinel::start_sentinel,
    syncer::syncer_loop,
};

#[macro_use]
extern crate log;

use std::result::Result;

use clap::Parser;
use lib::{init_logger, LogLevel, SentinelConfig, SentinelError};
use serde_json::json;

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

    let r = start_sentinel(
        &config,
        cli_args.disable_native_syncer,
        cli_args.disable_host_syncer,
        cli_args.disable_broadcaster,
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
