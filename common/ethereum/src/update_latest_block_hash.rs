use common::{traits::DatabaseInterface, types::Result};

use crate::{eth_database_utils::EthDbUtilsExt, eth_submission_material::EthSubmissionMaterial, EthState};

pub fn update_latest_block_hash_if_subsequent<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    sub_mat: &EthSubmissionMaterial,
) -> Result<()> {
    info!(
        "✔ Updating latest {} block hash if subsequent...",
        if db_utils.get_is_for_eth() { "ETH" } else { "EVM" }
    );
    db_utils
        .get_eth_latest_block_from_db()
        .and_then(|latest_submission_material| latest_submission_material.get_block_number())
        .and_then(|latest_block_number| {
            if latest_block_number + 1 == sub_mat.get_block_number()? {
                info!("✔ Block IS subsequent ∴ updating latest block hash...",);
                db_utils.put_eth_latest_block_hash_in_db(&sub_mat.get_block_hash()?)
            } else {
                info!("✔ Block NOT subsequent ∴ NOT updating latest block hash!");
                Ok(())
            }
        })
}

fn maybe_update_latest_block_hash_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe updating latest ETH block hash if subsequent...");
    if is_for_eth {
        update_latest_block_hash_if_subsequent(&state.eth_db_utils, state.get_eth_submission_material()?).and(Ok(state))
    } else {
        update_latest_block_hash_if_subsequent(&state.evm_db_utils, state.get_eth_submission_material()?).and(Ok(state))
    }
}

pub fn maybe_update_latest_eth_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_update_latest_block_hash_and_return_state(true, state)
}

pub fn maybe_update_latest_evm_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_update_latest_block_hash_and_return_state(false, state)
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{
        eth_database_utils::EthDbUtils,
        eth_types::EthHash,
        test_utils::{
            get_eth_latest_block_hash_from_db,
            get_sequential_eth_blocks_and_receipts,
            put_eth_latest_block_in_db,
        },
    };

    #[test]
    fn should_update_latest_block_hash_if_subsequent() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let latest_submission_material = get_sequential_eth_blocks_and_receipts()[0].clone();
        let latest_block_hash_before = latest_submission_material.get_block_hash().unwrap();
        put_eth_latest_block_in_db(&eth_db_utils, &latest_submission_material).unwrap();
        let subsequent_submission_material = get_sequential_eth_blocks_and_receipts()[1].clone();
        let expected_block_hash_after = subsequent_submission_material.get_block_hash().unwrap();
        update_latest_block_hash_if_subsequent(&eth_db_utils, &subsequent_submission_material).unwrap();
        let latest_block_hash_after = get_eth_latest_block_hash_from_db(&eth_db_utils).unwrap();
        assert_ne!(latest_block_hash_before, latest_block_hash_after);
        assert_eq!(latest_block_hash_after, expected_block_hash_after);
    }

    #[test]
    fn should_not_update_latest_block_hash_if_not_subsequent() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let latest_submission_material = get_sequential_eth_blocks_and_receipts()[0].clone();
        let latest_block_hash_before = latest_submission_material.get_block_hash().unwrap();
        put_eth_latest_block_in_db(&eth_db_utils, &latest_submission_material).unwrap();
        let non_subsequent_submission_material = get_sequential_eth_blocks_and_receipts()[0].clone();
        update_latest_block_hash_if_subsequent(&eth_db_utils, &non_subsequent_submission_material).unwrap();
        let latest_block_hash_after = eth_db_utils
            .get_hash_from_db_via_hash_key(EthHash::from_slice(&eth_db_utils.get_eth_latest_block_hash_key()[..]))
            .unwrap()
            .unwrap();
        assert_eq!(latest_block_hash_before, latest_block_hash_after);
    }
}
