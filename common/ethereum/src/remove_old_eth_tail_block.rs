use common::{traits::DatabaseInterface, types::Result};
use ethereum_types::H256 as EthHash;

use crate::{eth_database_utils::EthDbUtilsExt, eth_submission_material::EthSubmissionMaterial, EthState};

fn is_anchor_block<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E, eth_block_hash: &EthHash) -> Result<bool> {
    match db_utils.get_eth_anchor_block_hash_from_db() {
        Ok(ref hash) => Ok(hash == eth_block_hash),
        _ => Err("✘ No anchor hash found in db!".into()),
    }
}

pub fn remove_parents_if_not_anchor<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    block_whose_parents_to_be_removed: &EthSubmissionMaterial,
) -> Result<()> {
    match db_utils.get_submission_material_from_db(&block_whose_parents_to_be_removed.get_parent_hash()?) {
        Err(_) => {
            info!("✔ No block found ∵ doing nothing!");
            Ok(())
        },
        Ok(parent_block) => {
            info!("✔ Block found, checking if it's the anchor block...");
            if is_anchor_block(db_utils, &parent_block.get_block_hash()?)? {
                info!("✔ Block IS the anchor block ∴ not removing it!");
                Ok(())
            } else {
                info!("✔ Block is NOT the anchor ∴ removing it...");
                db_utils
                    .delete_block_by_block_hash(&parent_block)
                    .and_then(|_| remove_parents_if_not_anchor(db_utils, &parent_block))
            }
        },
    }
}

pub fn maybe_remove_tail_and_older_blocks<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<()> {
    let is_for_eth = db_utils.get_is_for_eth();
    info!(
        "✔ Maybe removing old {} tail block...",
        if is_for_eth { "ETH" } else { "EVM" }
    );
    db_utils
        .get_eth_tail_block_from_db()
        .and_then(|ref tail| remove_parents_if_not_anchor(db_utils, tail))
        .and(Ok(()))
}

pub fn maybe_remove_old_evm_tail_block_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_remove_tail_and_older_blocks(&state.evm_db_utils).and(Ok(state))
}

pub fn maybe_remove_old_eth_tail_block_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_remove_tail_and_older_blocks(&state.eth_db_utils).and(Ok(state))
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{
        eth_database_utils::EthDbUtils,
        test_utils::{get_sequential_eth_blocks_and_receipts, put_eth_anchor_block_in_db, put_eth_tail_block_in_db},
    };

    #[test]
    fn should_return_false_block_is_not_anchor_block() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let anchor_block = blocks[0].clone();
        let non_anchor_block = blocks[1].clone();
        assert_ne!(anchor_block, non_anchor_block);
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        let result = is_anchor_block(&eth_db_utils, &non_anchor_block.get_block_hash().unwrap()).unwrap();
        assert!(!result);
    }

    #[test]
    fn should_return_true_if_block_is_anchor_block() {
        let db = get_test_database();
        let blocks = get_sequential_eth_blocks_and_receipts();
        let eth_db_utils = EthDbUtils::new(&db);
        let anchor_block = blocks[0].clone();
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        let result = is_anchor_block(&eth_db_utils, &anchor_block.get_block_hash().unwrap()).unwrap();
        assert!(result);
    }

    #[test]
    fn should_remove_parent_block_if_parent_is_not_anchor() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let anchor_block = blocks[0].clone();
        let block = blocks[2].clone();
        let parent_block = blocks[1].clone();
        assert_eq!(parent_block.get_block_hash().unwrap(), block.get_parent_hash().unwrap());
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&parent_block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&parent_block.get_block_hash().unwrap()));
        remove_parents_if_not_anchor(&eth_db_utils, &block).unwrap();
        assert!(!eth_db_utils.eth_block_exists_in_db(&parent_block.get_block_hash().unwrap()));
    }

    #[test]
    fn should_not_remove_parent_block_if_parent_is_anchor() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let anchor_block = blocks[0].clone();
        let block = blocks[1].clone();
        assert_eq!(block.get_parent_hash().unwrap(), anchor_block.get_block_hash().unwrap());
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&anchor_block.get_block_hash().unwrap()));
        remove_parents_if_not_anchor(&eth_db_utils, &block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&block.get_block_hash().unwrap()));
    }

    #[test]
    fn should_remove_parent_blocks_recursively_if_not_anchor_blocks() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let all_blocks = get_sequential_eth_blocks_and_receipts();
        let blocks = &all_blocks[1..all_blocks.len() - 1];
        let tail_block = all_blocks[all_blocks.len() - 1].clone();
        let anchor_block = all_blocks[0].clone();
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        put_eth_tail_block_in_db(&eth_db_utils, &tail_block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&anchor_block.get_block_hash().unwrap()));
        blocks
            .iter()
            .try_for_each(|block| eth_db_utils.put_eth_submission_material_in_db(block))
            .unwrap();
        blocks
            .iter()
            .for_each(|block| assert!(eth_db_utils.eth_block_exists_in_db(&block.get_block_hash().unwrap())));
        remove_parents_if_not_anchor(&eth_db_utils, &tail_block).unwrap();
        blocks
            .iter()
            .for_each(|block| assert!(!eth_db_utils.eth_block_exists_in_db(&block.get_block_hash().unwrap())));
        assert!(eth_db_utils.eth_block_exists_in_db(&tail_block.get_block_hash().unwrap()));
        assert!(eth_db_utils.eth_block_exists_in_db(&anchor_block.get_block_hash().unwrap()));
    }
}
