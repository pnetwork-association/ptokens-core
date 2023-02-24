use anyhow::Result;
use clap::Args;
use lib::{get_rpc_client, get_sub_mat, Endpoints};
use serde_json::json;

use crate::cli::write_file;

#[derive(Debug, Subcommand)]
pub enum GetSubMatSubCommand {
    /// Get HOST submission material.
    GetHostSubMat(SubMatGetterArgs),

    /// Get NATIVE submission material.
    GetNativeSubMat(SubMatGetterArgs),
}

#[derive(Debug, Args)]
pub struct SubMatGetterArgs {
    /// Block number to create the submission material for.
    pub block_num: u64,

    /// Optional path to save the submission material to.
    #[arg(long, short)]
    pub path: Option<String>,
}

async fn get_sub_mat_cli(endpoints: &Endpoints, args: &SubMatGetterArgs, is_native: bool) -> Result<String> {
    let endpoint = endpoints.get_first_endpoint(is_native)?;
    let ws_client = get_rpc_client(&endpoint).await?;
    let sub_mat_type = if is_native { "native" } else { "host" };
    info!("[+] Getting {sub_mat_type} submission material...");
    let sub_mat = get_sub_mat(&ws_client, args.block_num).await?;

    // FIXME TODO get from config
    let block_is_valid = sub_mat.block_is_valid(&common_chain_ids::EthChainId::Mainnet)?;

    let receipts_are_valid = sub_mat.receipts_are_valid()?;
    if !block_is_valid {
        return Err(anyhow!("Block header is not valid!"));
    }
    let merkle_root = sub_mat.receipts.get_merkle_root().unwrap();
    if !receipts_are_valid {
        println!("Invalid receipts root: {}", merkle_root);
        return Err(anyhow!("Receipts are not valid!"));
    };
    let block_num = sub_mat.get_block_number()?;
    let s = serde_json::to_string(&sub_mat)?;
    let path = args.path.clone().unwrap_or_else(|| ".".into());
    let full_path = format!("{path}/{sub_mat_type}-sub-mat-num-{block_num}.json");
    write_file(&s, &full_path)?;
    Ok(json!({ "jsonrpc": "2.0", "result": full_path }).to_string())
}

pub async fn get_native_sub_mat(endpoints: &Endpoints, args: &SubMatGetterArgs) -> Result<String> {
    get_sub_mat_cli(endpoints, args, true).await
}

pub async fn get_host_sub_mat(endpoints: &Endpoints, args: &SubMatGetterArgs) -> Result<String> {
    get_sub_mat_cli(endpoints, args, false).await
}
