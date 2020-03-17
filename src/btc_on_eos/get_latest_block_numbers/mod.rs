use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    btc::btc_database_utils::get_btc_latest_block_number,
    check_core_is_initialized::check_btc_core_is_initialized,
};

#[derive(Serialize, Deserialize)]
pub struct BlockNumbers {
    btc_latest_block_number: u64,
}

pub fn get_latest_block_numbers<D>(
    db: D,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Getting latest block numbers...");
    check_btc_core_is_initialized(&db)
        .and_then(|_| {
            Ok(serde_json::to_string(
                &BlockNumbers {
                    btc_latest_block_number: get_btc_latest_block_number(&db)?,
                }
            )?)
        })
}
