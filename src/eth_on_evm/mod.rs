#![allow(unused_imports)] // FIXME RM!
#![allow(dead_code)] // FIXME RM!

//! # The `pETH-on-EVM` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native ETH tokens and their ERC777 pToken equivalents on EVM compliant chains.
//! This core consists of two light clients that manage the state of the two chains,
//! along with the creation and signing of transactions related to each chain.
//!
//! __NOTE:__ All `debug_` prefixed functions can only be used if the core is
//! built with the `debug` feaure flag enabled in the `Cargo.toml`:
//!
//! ```no_compile
//! ptokens_core = { version = "4.5.0", features = ["debug"] }
//! ```

pub mod debug_functions;
pub mod eth;
pub mod evm;
pub mod get_enclave_state;
pub mod get_latest_block_numbers;

pub(crate) mod check_core_is_initialized;

pub use crate::{
    chains::{
        eth::eth_message_signer::{sign_ascii_msg_with_eth_key_with_no_prefix, sign_hex_msg_with_eth_key_with_prefix},
        evm::eth_message_signer::{
            sign_ascii_msg_with_eth_key_with_no_prefix as sign_ascii_msg_with_evm_key_with_no_prefix,
            sign_hex_msg_with_eth_key_with_prefix as sign_hex_msg_with_evm_key_with_prefix,
        },
    },
    eth_on_evm::{
        debug_functions::{
            debug_get_all_db_keys,
            debug_get_key_from_db,
            debug_reprocess_eth_block,
            debug_reprocess_evm_block,
            debug_set_key_in_db_to_value,
        },
        eth::{initialize_eth_core::maybe_initialize_eth_core, submit_eth_block::submit_eth_block_to_core},
        evm::{initialize_evm_core::maybe_initialize_evm_core, submit_evm_block::submit_evm_block_to_core},
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
    },
};
