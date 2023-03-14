use common::{errors::AppError, traits::DatabaseInterface, types::Result, BridgeSide, NoParentError};
use ethereum_types::U256;

use crate::{eth_database_utils::EthDbUtilsExt, EthState, EthSubmissionMaterial};

fn get_no_parent_error(n: U256, bridge_side: BridgeSide) -> AppError {
    AppError::NoParentError(NoParentError::new(
        n.as_u64(),
        format!("✘ {bridge_side} block #{n} rejected - no parent exists in database!"),
        bridge_side,
    ))
}

pub fn check_for_parent_of_block<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    sub_mat: &EthSubmissionMaterial,
) -> Result<()> {
    let bridge_side = if db_utils.get_is_for_eth() {
        BridgeSide::Native
    } else {
        BridgeSide::Host
    };
    let parent_hash = sub_mat.get_parent_hash()?;
    let block_number = sub_mat.get_block_number()?;

    info!("✔ Checking if {bridge_side} block #{block_number}'s parent exists in database...");
    if db_utils.eth_block_exists_in_db(&parent_hash) {
        info!("✔ {bridge_side} block's parent exists in database!");
        Ok(())
    } else {
        Err(get_no_parent_error(block_number, bridge_side))
    }
}

fn check_for_parent_of_block_in_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    let bridge_side = if is_for_eth {
        BridgeSide::Native
    } else {
        BridgeSide::Host
    };
    let parent_hash = state.get_parent_hash()?;
    let block_number = state.get_block_num()?;

    info!("✔ Checking if {bridge_side} block #{block_number}'s parent exists in database...");
    let parent_exists = if is_for_eth {
        state.eth_db_utils.eth_block_exists_in_db(&parent_hash)
    } else {
        state.evm_db_utils.eth_block_exists_in_db(&parent_hash)
    };
    if parent_exists {
        info!("✔ {bridge_side} block's parent exists in database!");
        Ok(state)
    } else {
        Err(get_no_parent_error(block_number, bridge_side))
    }
}

pub fn check_for_parent_of_eth_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_for_parent_of_block_in_state(true, state)
}

pub fn check_for_parent_of_evm_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_for_parent_of_block_in_state(false, state)
}
