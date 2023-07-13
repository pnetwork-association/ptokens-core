mod cli;
mod handle_cli;
mod sentinel;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use crate::handle_cli::handle_cli;

#[tokio::main]
async fn main() {
    match handle_cli().await {
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
