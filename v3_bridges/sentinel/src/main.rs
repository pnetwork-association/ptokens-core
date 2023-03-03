mod cli;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use anyhow::Result;
use clap::Parser;
use cli::{
    get_latest_block_num::{get_host_latest_block_num, get_native_latest_block_num},
    get_sub_mat::{get_host_sub_mat, get_native_sub_mat},
    CliArgs,
    SubCommands,
};
use futures::join;
use lib::{get_rpc_client, get_sub_mat, SentinelConfig, SubMatBatch};
use serde_json::json;

async fn do_thing(mut batch: SubMatBatch) -> Result<String> {
    let ws_client = batch.get_rpc_client().await?;
    let mut block_num = 16742150;
    loop {
        let block = get_sub_mat(&ws_client, block_num).await?;
        batch.push(block);
        if batch.is_ready_to_submit() {
            info!("Batch is ready to submit!");
            break;
        } else {
            block_num += 1;
        }
    }
    Ok(format!("{}_success", if batch.is_native() { "native" } else { "host" }).into())
}

#[tokio::main]
async fn main() {
    let config = SentinelConfig::new().unwrap();

    let cli_args = CliArgs::parse();

    let r = match cli_args.sub_commands {
        SubCommands::Start => {
            let batch_1 = SubMatBatch::new_from_config(true, &config).unwrap();
            let batch_2 = SubMatBatch::new_from_config(false, &config).unwrap();
            assert!(batch_1.is_native(), "Batch 1 is NOT native!");
            assert!(batch_2.is_host(), "Batch 2 is NOT host!");

            let thread_1 = tokio::spawn(async move { do_thing(batch_1).await });
            let thread_2 = tokio::spawn(async move { do_thing(batch_2).await });

            use futures::try_join; // NOTE: Use me to end early on an Err in one of the threads! Or
                                   // look into JoinSet which allows tasks to be aborted
            let (res_1, res_2) = join!(thread_1, thread_2);
            let thread_1_result = res_1.unwrap().unwrap();
            let thread_2_result = res_2.unwrap().unwrap();
            let res = json!({
                "jsonrpc": "2.0",
                "result": {
                    "thread_1": thread_1_result,
                    "thread_2": thread_2_result,
                },
            })
            .to_string();
            Ok(res)
        },
        SubCommands::GetHostSubMat(ref args) => get_host_sub_mat(&config.host.get_endpoints(), args).await,
        SubCommands::GetNativeSubMat(ref args) => get_native_sub_mat(&config.native.get_endpoints(), args).await,
        SubCommands::GetHostLatestBlockNum => get_host_latest_block_num(&config.host.get_endpoints()).await,
        SubCommands::GetNativeLatestBlockNum => get_native_latest_block_num(&config.native.get_endpoints()).await,
    };

    match r {
        Ok(res) => println!("{res}"),
        Err(err) => {
            println!("{}", json!({"jsonrpc": "2.0", "error": err.to_string()}));
            std::process::exit(1)
        },
    };
}

// TODO use https://crates.io/crates/async-log for async logging when we have > 1 thread (flexi
// logger can do it apparently)
// JSON-RPC spec:https://www.jsonrpc.org/specificationhttps://www.jsonrpc.org/specification https://www.jsonrpc.org/specification
