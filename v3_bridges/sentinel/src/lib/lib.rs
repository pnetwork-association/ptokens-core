mod check_endpoint;
mod constants;
mod errors;
mod get_block;
mod get_config;
mod get_latest_block_number;
mod get_receipts;
mod get_rpc_client;
mod get_sub_mat;
mod test_utils;

pub use self::{
    check_endpoint::check_endpoint,
    errors::SentinelError,
    get_block::get_block,
    get_config::{Config, Endpoints},
    get_latest_block_number::get_latest_block_number,
    get_receipts::get_receipts,
    get_rpc_client::get_rpc_client,
    get_sub_mat::get_sub_mat,
};

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
