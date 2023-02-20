mod cli;
mod lib;

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

use clap::Parser;
use cli::{
    get_submission_material::{
        get_host_submission_material,
        get_native_submission_material,
        GetSubmissionMaterialSubCommand,
    },
    CliArgs,
};
use lib::{get_block::get_block, get_config::Config, get_receipts::get_receipts, get_rpc_client::get_rpc_client};

#[tokio::main]
async fn main() {
    let config = Config::new().unwrap();
    println!("config: {config:?}");

    use simple_logger; // FIXME rm!
    simple_logger::init_with_level(config.get_log_level()).unwrap(); // FIXME rm!

    let cli_args = CliArgs::parse();

    match cli_args.get_submission_material {
        GetSubmissionMaterialSubCommand::GetHostSubmissionMaterial(_args) => {
            get_host_submission_material().unwrap();
        },
        GetSubmissionMaterialSubCommand::GetNativeSubmissionMaterial(_args) => {
            get_native_submission_material().unwrap();
        },
    };

    /*
    let ws_client = get_rpc_client(&config.endpoints.host[0]).await.unwrap();
    for i in 0..10 {
        let block = get_block(&ws_client, 16640614 + i).await.unwrap();
        let sub_mat = get_receipts(&ws_client, block).await.unwrap();
        let receipts_are_valid = sub_mat.receipts_are_valid().unwrap();
        warn!(
            "[+] {} receipts are valid: {receipts_are_valid}",
            sub_mat.receipts.len()
        )
    }
    */
}

// TODO use https://crates.io/crates/async-log for async logging when we have > 1 thread
// TODO use confy for easy config management, and toml files - better than json here.
// Write logs to a file, and watch that during running. (see vanilla apps for how that's done)
// pull the vanilla apps into this workspace too to share deps (rocksdb mostly)
// JSON-RPC spec:https://www.jsonrpc.org/specificationhttps://www.jsonrpc.org/specification https://www.jsonrpc.org/specification
