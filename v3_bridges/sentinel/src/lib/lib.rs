mod check_endpoint;
mod constants;
mod errors;
mod get_block;
mod get_config;
mod get_latest_block_num;
mod get_receipts;
mod get_rpc_client;
mod get_sub_mat;
mod test_utils;

pub use self::{
    check_endpoint::check_endpoint,
    errors::SentinelError,
    get_block::get_block,
    get_config::{Config, Endpoints},
    get_latest_block_num::get_latest_block_num,
    get_receipts::get_receipts,
    get_rpc_client::get_rpc_client,
    get_sub_mat::get_sub_mat,
};

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
