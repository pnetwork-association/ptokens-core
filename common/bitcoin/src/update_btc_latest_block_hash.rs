use common::{traits::DatabaseInterface, types::Result};

use crate::BtcState;

fn is_block_subsequent(block_in_question_height: u64, latest_block_from_database_height: u64) -> bool {
    latest_block_from_database_height == block_in_question_height + 1
}

pub fn maybe_update_btc_latest_block_hash<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    state
        .btc_db_utils
        .get_btc_latest_block_from_db()
        .and_then(|latest_block_and_id| {
            match is_block_subsequent(latest_block_and_id.height, state.get_btc_block_and_id()?.height) {
                false => {
                    info!("✔ BTC block NOT subsequent {}", "∴ NOT updating latest block hash",);
                    Ok(state)
                },
                true => {
                    info!("✔ BTC block IS subsequent {}", "∴ updating latest block hash...",);
                    state
                        .btc_db_utils
                        .put_btc_latest_block_hash_in_db(&state.get_btc_block_and_id()?.id)
                        .map(|_| state)
                },
            }
        })
}
