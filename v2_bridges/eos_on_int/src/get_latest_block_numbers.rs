use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eos::EosDbUtils;
#[cfg(not(feature = "spring_1-0"))]
use common_eos::Incremerkles;
use common_eth::{EthDbUtils, EthDbUtilsExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    int_latest_block_number: usize,
    eos_latest_block_number: u64,
    #[cfg(not(feature = "spring_1-0"))]
    eos_previous_block_numbers: Vec<u64>,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting latest block numbers...");
    #[cfg(not(feature = "spring_1-0"))]
    let incremerkles = Incremerkles::get_from_db(&EosDbUtils::new(db))?;

    CoreType::check_is_initialized(db).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            #[cfg(feature = "spring_1-0")]
            eos_latest_block_number: EosDbUtils::new(db).get_latest_eos_block_number()?,
            #[cfg(not(feature = "spring_1-0"))]
            eos_latest_block_number: incremerkles.latest_block_num(),
            #[cfg(not(feature = "spring_1-0"))]
            eos_previous_block_numbers: incremerkles.previous_block_nums(),
            int_latest_block_number: EthDbUtils::new(db).get_latest_eth_block_number()?,
        })?)
    })
}
