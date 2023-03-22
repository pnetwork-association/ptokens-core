#![recursion_limit = "256"] // NOTE: Because of the error macro.
#![allow(clippy::too_many_arguments)]

//! # The __`pToken`__ Core
//!
//! Herein lies the functionality required for the cross-chain conversions
//! between various blockchains allowing for decentalized swaps between a native
//! asset and a host chain's pTokenized version of that asset.
//!
//! __Note:__ When compiling the core, you may provide an optional environment
//! variable __`DB_KEY_PREFIX`__, which when used will prefix all database keys
//! with the provided argument. Via this, database key clashes can be avoided
//! if running multiple instances on one machine.

pub use crate::{
    block_already_in_db_error::BlockAlreadyInDbError,
    bridge_side::BridgeSide,
    core_type::{CoreType, V3CoreType},
    errors::AppError,
    no_parent_error::NoParentError,
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::{strip_hex_prefix, get_core_version},
};

// FIXME Sort out the pub mods
#[macro_use]
pub mod macros;
pub mod address;
mod block_already_in_db_error;
mod bridge_side;
pub mod constants;
pub mod core_type;
pub mod crypto_utils;
pub mod dictionaries;
pub mod errors;
mod no_parent_error;
pub mod test_utils;
pub mod traits;
pub mod types;
pub mod utils;

#[cfg(test)]
extern crate simple_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate paste;
