#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

mod cli;
mod cmds;
mod handle_cli;
mod write_file;

#[tokio::main]
async fn main() {
    match self::handle_cli::handle_cli().await {
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
