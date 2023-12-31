use common::{traits::DatabaseInterface, types::Result};

use crate::BtcState;

pub fn maybe_set_btc_latest_block_hash<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Checking BTC latest block hash is set in database...");
    if state
        .btc_db_utils
        .key_exists_in_db(&state.btc_db_utils.get_btc_latest_block_hash_key(), None)
    {
        info!("✔ BTC latest block hash set in database");
        Ok(state)
    } else {
        info!("✔ Initializing BTC latest block hash from in block...");
        state
            .btc_db_utils
            .put_btc_latest_block_hash_in_db(&state.get_btc_block_and_id()?.id)
            .and(Ok(state))
    }
}
