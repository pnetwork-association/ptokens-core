mod check_endpoint;
mod config;
mod constants;
mod errors;
mod get_block;
mod get_latest_block_num;
mod get_receipts;
mod get_rpc_client;
mod get_sub_mat;
mod logging;
mod sub_mat_batch;
mod test_utils;

pub use self::{
    check_endpoint::check_endpoint,
    config::{Config as SentinelConfig, Endpoints},
    errors::SentinelError,
    get_block::get_block,
    get_latest_block_num::get_latest_block_num,
    get_receipts::get_receipts,
    get_rpc_client::get_rpc_client,
    get_sub_mat::get_sub_mat,
    sub_mat_batch::SubMatBatch,
};

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
