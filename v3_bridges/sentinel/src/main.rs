mod cli;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use clap::Parser;
use cli::{
    get_sub_mat::{get_host_sub_mat, get_native_sub_mat, GetSubMatSubCommand},
    CliArgs,
};
use lib::Config;
use serde_json::json;

#[tokio::main]
async fn main() {
    let config = Config::new().unwrap();
    //println!("config: {config:?}");

    //let ws_client = get_rpc_client(&config.endpoints.host[0]).await.unwrap();

    //use simple_logger; // FIXME rm!
    //simple_logger::init_with_level(config.get_log_level()).unwrap(); // FIXME rm!

    let cli_args = CliArgs::parse();

    let r = match cli_args.get_sub_mat {
        GetSubMatSubCommand::GetHostSubMat(ref args) => get_host_sub_mat(&config.endpoints, args).await,
        GetSubMatSubCommand::GetNativeSubMat(ref args) => get_native_sub_mat(&config.endpoints, args).await,
    };

    match r {
        Ok(res) => println!("{res}"),
        Err(err) => println!("{}", json!({"jsonrpc": "2.0", "error": err.to_string()})),
    };
}

// TODO use https://crates.io/crates/async-log for async logging when we have > 1 thread
// TODO use confy for easy config management, and toml files - better than json here.
// Write logs to a file, and watch that during running. (see vanilla apps for how that's done)
// pull the vanilla apps into this workspace too to share deps (rocksdb mostly)
// JSON-RPC spec:https://www.jsonrpc.org/specificationhttps://www.jsonrpc.org/specification https://www.jsonrpc.org/specification
