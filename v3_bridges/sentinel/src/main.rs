mod cli;
mod handle_cli;
mod sentinel;
mod syncer;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use serde_json::json;

use crate::handle_cli::handle_cli;

#[tokio::main]
async fn main() {
    match handle_cli().await {
        Ok(s) => {
            info!("{s}");
            println!("{s}");
        },
        Err(err) => {
            let s = format!("{}", json!({"jsonrpc": "2.0", "error": err.to_string()}));
            info!("{s}");
            eprintln!("{s}");
            std::process::exit(1)
        },
    };
}
