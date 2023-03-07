mod batching;
mod config;
mod constants;
mod endpoints;
mod error;
mod get_block;
mod get_latest_block_num;
mod get_receipts;
mod get_rpc_client;
mod get_sub_mat;
mod logging;
mod test_utils;

pub use self::{
    batching::SubMatBatch,
    config::Config as SentinelConfig,
    constants::MILLISECONDS_MULTIPLIER,
    endpoints::{check_endpoint, Endpoints},
    error::SentinelError,
    get_block::get_block,
    get_latest_block_num::get_latest_block_num,
    get_receipts::get_receipts,
    get_rpc_client::get_rpc_client,
    get_sub_mat::get_sub_mat,
    logging::init_logger,
};

#[macro_use]
extern crate log;
