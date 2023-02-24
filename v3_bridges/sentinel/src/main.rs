mod cli;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use clap::Parser;
use cli::{
    get_latest_block_num::{get_host_latest_block_num, get_native_latest_block_num},
    get_sub_mat::{get_host_sub_mat, get_native_sub_mat},
    CliArgs,
    SubCommands,
};
use lib::Config;
use serde_json::json;

#[tokio::main]
async fn main() {
    let config = Config::new().unwrap();
    use simple_logger; // FIXME rm!
    simple_logger::init_with_level(config.get_log_level()).unwrap(); // FIXME rm!

    let cli_args = CliArgs::parse();

    let r = match cli_args.sub_commands {
        SubCommands::GetHostSubMat(ref args) => get_host_sub_mat(&config.endpoints, args).await,
        SubCommands::GetNativeSubMat(ref args) => get_native_sub_mat(&config.endpoints, args).await,
        SubCommands::GetHostLatestBlockNum => get_host_latest_block_num(&config.endpoints).await,
        SubCommands::GetNativeLatestBlockNum => get_native_latest_block_num(&config.endpoints).await,
    };

    match r {
        Ok(res) => println!("{res}"),
        Err(err) => {
            println!("{}", json!({"jsonrpc": "2.0", "error": err.to_string()}));
            std::process::exit(1)
        },
    };
}

// TODO use https://crates.io/crates/async-log for async logging when we have > 1 thread
// Write logs to a file, and watch that during running. (see vanilla apps for how that's done)
// pull the vanilla apps into this workspace too to share deps (rocksdb mostly)
// JSON-RPC spec:https://www.jsonrpc.org/specificationhttps://www.jsonrpc.org/specification https://www.jsonrpc.org/specification
