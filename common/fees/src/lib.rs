// NOTE: This means we can (relatively) easily make this LTC based instead of BTC.
#[cfg(not(feature = "ltc"))]
use bitcoin as bitcoin_crate_alias;
#[cfg(feature = "ltc")]
use litecoin as bitcoin_crate_alias;

mod fee_constants;
mod fee_database_utils;
mod fee_enclave_state;
mod fee_utils;
mod fee_withdrawals;
mod test_utils;

pub use self::{
    fee_constants::{BTC_ON_ETH_FEE_DB_KEYS, DISABLE_FEES, FEE_BASIS_POINTS_DIVISOR},
    fee_database_utils::FeeDatabaseUtils,
    fee_enclave_state::FeesEnclaveState,
    fee_utils::sanity_check_basis_points_value,
    fee_withdrawals::{get_btc_on_eos_fee_withdrawal_tx, get_btc_on_eth_fee_withdrawal_tx},
};

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
