mod core_config;
mod error;

#[macro_use]
extern crate log;

use std::result::Result;

use clap::Parser;
use lib::{init_logger, LogLevel};
use serde_json::json;

use self::{core_config::CoreConfig, error::CoreError};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Log level - if extant, this overrides log level set in config
    #[arg(long, short)]
    log_level: Option<LogLevel>,
}

impl Cli {
    pub fn log_level(&self) -> Option<log::Level> {
        self.log_level.map(|l| l.into())
    }
}

async fn start() -> Result<String, CoreError> {
    let config = CoreConfig::new()?;

    let cli_args = Cli::parse();

    if config.log().is_enabled() {
        init_logger(&config.log(), cli_args.log_level())?
    };

    let r: Result<String, CoreError> = Ok("this does nothing".into());
    r.map_err(|e| CoreError::Json(json!({"jsonrpc": "2.0", "error": e.to_string()})))
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
