use common::{traits::DatabaseInterface, types::Result, AppError, BlockAlreadyInDbError, BridgeSide};

use crate::{eth_database_utils::EthDbUtilsExt, eth_state::EthState, eth_submission_material::EthSubmissionMaterial};

pub fn add_block_and_receipts_to_db_if_not_extant<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    block_and_receipts: &EthSubmissionMaterial,
) -> Result<()> {
    let is_for_eth = db_utils.get_is_for_eth();
    info!(
        "✔ Adding {} block and receipts if not already in db...",
        if is_for_eth { "ETH" } else { "EVM" }
    );
    if db_utils.eth_block_exists_in_db(&block_and_receipts.get_block_hash()?) {
        let e = BlockAlreadyInDbError::new(
            block_and_receipts.get_block_number()?.as_u64(),
            "✘ Block Rejected - it's already in the db!".to_string(),
            if is_for_eth {
                BridgeSide::Native
            } else {
                BridgeSide::Host
            },
        );
        Err(AppError::BlockAlreadyInDbError(e))
    } else {
        info!("✔ Block & receipts not in db, adding them now...");
        db_utils.put_eth_submission_material_in_db(block_and_receipts)
    }
}

fn maybe_add_block_and_receipts_to_db_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!(
        "✔ Maybe adding {} block and receipts if not in db...",
        if is_for_eth { "ETH" } else { "EVM" }
    );
    let submission_material = state.get_eth_submission_material()?;
    if is_for_eth {
        add_block_and_receipts_to_db_if_not_extant(&state.eth_db_utils, submission_material).and(Ok(state))
    } else {
        add_block_and_receipts_to_db_if_not_extant(&state.evm_db_utils, submission_material).and(Ok(state))
    }
}

pub fn maybe_add_eth_block_and_receipts_to_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_add_block_and_receipts_to_db_and_return_state(true, state)
}

pub fn maybe_add_evm_block_and_receipts_to_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_add_block_and_receipts_to_db_and_return_state(false, state)
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{eth_database_utils::EthDbUtils, test_utils::get_sample_eth_submission_material_n};

    #[test]
    fn should_maybe_add_block_and_receipts_to_db() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let block_and_receipts = get_sample_eth_submission_material_n(1).unwrap();
        let eth_block_hash = block_and_receipts.get_block_hash().unwrap();
        let bool_before = eth_db_utils.eth_block_exists_in_db(&eth_block_hash);
        assert!(!bool_before);
        add_block_and_receipts_to_db_if_not_extant(&eth_db_utils, &block_and_receipts).unwrap();
        let bool_after = eth_db_utils.eth_block_exists_in_db(&eth_block_hash);
        assert!(bool_after);
    }

    #[test]
    fn should_error_if_block_already_in_db() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let block_and_receipts = get_sample_eth_submission_material_n(1).unwrap();
        let eth_block_hash = block_and_receipts.get_block_hash().unwrap();
        let bool_before = eth_db_utils.eth_block_exists_in_db(&eth_block_hash);
        assert!(!bool_before);
        add_block_and_receipts_to_db_if_not_extant(&eth_db_utils, &block_and_receipts).unwrap();
        let bool_after = eth_db_utils.eth_block_exists_in_db(&eth_block_hash);
        assert!(add_block_and_receipts_to_db_if_not_extant(&eth_db_utils, &block_and_receipts).is_err());
        assert!(bool_after);
    }
}
