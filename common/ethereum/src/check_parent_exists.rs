use common::{errors::AppError, traits::DatabaseInterface, types::Result};
use serde_json::json;

use crate::{eth_database_utils::EthDbUtilsExt, EthState, EthSubmissionMaterial};

pub fn check_for_parent_of_block<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    sub_mat: &EthSubmissionMaterial,
) -> Result<()> {
    let block_type = if db_utils.get_is_for_eth() { "ETH" } else { "EVM" };
    let parent_hash = sub_mat.get_parent_hash()?;
    let block_number = sub_mat.get_block_number()?;
    info!("✔ Checking if {block_type} block #{block_number}'s parent exists in database...");
    if db_utils.eth_block_exists_in_db(&parent_hash) {
        info!("✔ {block_type} block's parent exists in database!");
        Ok(())
    } else {
        // TODO need a specific error type really for stuff like this :/
        Err(AppError::Json(json!({
            "blockNum": block_number.to_string(),
            "error": format!("✘ {block_type} block #{block_number} rejected - no parent exists in database!")
        })))
    }
}

fn check_for_parent_of_block_in_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    let block_type = if is_for_eth { "ETH" } else { "EVM" };
    let parent_hash = state.get_parent_hash()?;
    let block_number = state.get_block_num()?;
    info!(
        "✔ Checking if {} block #{}'s parent exists in database...",
        block_type, block_number
    );
    let parent_exists = if is_for_eth {
        state.eth_db_utils.eth_block_exists_in_db(&parent_hash)
    } else {
        state.evm_db_utils.eth_block_exists_in_db(&parent_hash)
    };
    if parent_exists {
        info!("✔ {} block's parent exists in database!", block_type);
        Ok(state)
    } else {
        Err(AppError::Json(json!({
            "blockNum": block_number.to_string(),
            "error": format!("✘ {} block #{} rejected - no parent exists in database!", block_type, block_number)
        })))
    }
}

pub fn check_for_parent_of_eth_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_for_parent_of_block_in_state(true, state)
}

pub fn check_for_parent_of_evm_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_for_parent_of_block_in_state(false, state)
}
