#![recursion_limit = "256"] // NOTE: Because of the error macro.

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

#![allow(clippy::match_bool)]
#![allow(clippy::too_many_arguments)]

pub use errors::AppError;
pub use types::{Bytes, Result};
pub use utils::get_core_version;

#[macro_use]
pub(crate) mod macros;
pub(crate) mod address;
pub mod btc_on_eos;
pub mod btc_on_eth;
pub mod btc_on_int;
pub mod chains;
pub(crate) mod core_type;
pub(crate) mod dictionaries;
pub mod eos_on_eth;
pub mod erc20_on_eos;
pub mod erc20_on_evm;
pub mod erc20_on_int;
pub mod errors;
pub(crate) mod fees;
pub mod int_on_algo;
pub mod int_on_eos;
pub mod int_on_evm;
pub(crate) mod metadata;
pub(crate) mod safe_addresses;
pub mod traits;
pub mod types;

mod check_debug_mode;
mod constants;
mod crypto_utils;
mod database_utils;
mod debug_database_utils;
mod enclave_info;
mod utils;

#[cfg(test)]
mod test_utils;
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
