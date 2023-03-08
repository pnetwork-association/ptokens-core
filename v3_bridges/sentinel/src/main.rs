mod cli;
mod syncer;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use anyhow::Result;
use clap::Parser;
use syncer::syncer_loop;
use cli::{
    get_latest_block_num::{get_host_latest_block_num, get_native_latest_block_num},
    get_sub_mat::{get_host_sub_mat, get_native_sub_mat},
    CliArgs,
    SubCommands,
};
use futures::join;
use lib::{init_logger, SentinelConfig, SubMatBatch};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let config = SentinelConfig::new()?;
    init_logger(&config.log_config)?;

    let cli_args = CliArgs::parse();

    let r = match cli_args.sub_commands {
        SubCommands::Start => {
            let batch_1 = SubMatBatch::new_from_config(true, &config)?;
            let batch_2 = SubMatBatch::new_from_config(false, &config)?;
            assert!(batch_1.is_native(), "Batch 1 is NOT native!");
            assert!(batch_2.is_host(), "Batch 2 is NOT host!");

            let thread_1 = tokio::spawn(async move { syncer_loop(batch_1).await });
            let thread_2 = tokio::spawn(async move { syncer_loop(batch_2).await });

            let (res_1, res_2) = join!(thread_1, thread_2);
            let thread_1_result = res_1??;
            let thread_2_result = res_2??;
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
        SubCommands::GetHostSubMat(ref args) => get_host_sub_mat(&config.host_config.get_endpoints(), args).await,
        SubCommands::GetNativeSubMat(ref args) => get_native_sub_mat(&config.native_config.get_endpoints(), args).await,
        SubCommands::GetHostLatestBlockNum => get_host_latest_block_num(&config.host_config.get_endpoints()).await,
        SubCommands::GetNativeLatestBlockNum => {
            get_native_latest_block_num(&config.native_config.get_endpoints()).await
        },
    };

    match r {
        Ok(s) => {
            println!("{s}");
            info!("{s}");
            Ok(())
        },
        Err(err) => {
            let s = format!("{}", json!({"jsonrpc": "2.0", "error": err.to_string()}));
            eprintln!("{s}");
            info!("{s}");
            std::process::exit(1)
        },
    }
}
// JSON-RPC spec:https://www.jsonrpc.org/specificationhttps://www.jsonrpc.org/specification https://www.jsonrpc.org/specification
// use futures::try_join; // NOTE: Use me to end early on an Err in one of the threads! Or look into JoinSet which
// allows tasks to be aborted
