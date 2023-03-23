use common::{traits::DatabaseInterface, types::Result};

use crate::{
    add_block_and_receipts_to_db::add_block_and_receipts_to_db_if_not_extant,
    check_parent_exists::check_for_parent_of_block,
    eth_database_utils::EthDbUtilsExt,
    eth_submission_material::EthSubmissionMaterial as EthSubMat,
    remove_old_eth_tail_block::maybe_remove_tail_and_older_blocks,
    update_eth_canon_block_hash::maybe_update_canon_block_hash,
    update_eth_linker_hash::maybe_update_linker_hash,
    update_eth_tail_block_hash::maybe_update_eth_tail_block_hash,
    update_latest_block_hash::update_latest_block_hash_if_subsequent,
};

/// Append to blockchain
///
/// A helper function that takes the db utils as an argument and appends the passed in submission
/// material to the chain in the database, if and only if it's valid & subsequent.
///
/// WARN: No checks are done on the receipt's validity, since at this point they'll likely have
/// bene truncated down to only pertinent ones in order to save space in the database.
pub fn append_to_blockchain<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    sub_mat: &EthSubMat,
    validate: bool,
) -> Result<()> {
    let n = sub_mat.get_block_number()?;
    let chain_id = db_utils.get_eth_chain_id_from_db()?;
    let confs = db_utils.get_eth_canon_to_tip_length_from_db()?;
    let header_is_valid = if validate {
        sub_mat.block_header_is_valid(&chain_id)
    } else {
        warn!("Block header validation is disabled!");
        true
    };
    if header_is_valid {
        info!("Adding block {n} to chain in db...");
        check_for_parent_of_block(db_utils, sub_mat)
            .and_then(|_| add_block_and_receipts_to_db_if_not_extant(db_utils, sub_mat))
            .and_then(|_| update_latest_block_hash_if_subsequent(db_utils, sub_mat))
            .and_then(|_| maybe_update_canon_block_hash(db_utils, confs))
            .and_then(|_| maybe_update_eth_tail_block_hash(db_utils))
            .and_then(|_| maybe_update_linker_hash(db_utils))
            .and_then(|_| maybe_remove_tail_and_older_blocks(db_utils))
            .and(Ok(()))
    } else {
        Err("Not adding block to database because it's not valid!".into())
    }
}

#[cfg(test)]
mod tests {
    use common::{errors::AppError, test_utils::get_test_database, BlockAlreadyInDbError, BridgeSide};
    use common_chain_ids::EthChainId;
    use ethereum_types::H256 as EthHash;

    use super::*;
    use crate::{
        calculate_linker_hash::calculate_linker_hash,
        eth_constants::ETH_TAIL_LENGTH,
        eth_database_utils::{EthDbUtils, SpecialHashes},
        initialize_eth_core_with_no_contract_tx,
        test_utils::get_sequential_eth_blocks_and_receipts,
        EthState,
    };

    #[test]
    fn should_append_to_blockchain_correctly() {
        let db = get_test_database();
        let blocks = get_sequential_eth_blocks_and_receipts();
        let block_hashes = blocks.iter().map(|b| b.get_block_hash().unwrap()).collect::<Vec<_>>();
        let mut last_submitted_block_idx = 0;
        let init_block = blocks[last_submitted_block_idx].clone();
        let state = EthState::init(&db);

        let db_utils = EthDbUtils::new(&db);
        let chain_id = EthChainId::InterimChain;
        let gas_price = 0;
        let confs: usize = 5;
        let is_native = true;
        let genesis_hash = EthHash::from_slice(&db_utils.get_eth_ptoken_genesis_hash_key());

        // Initialize the core...
        initialize_eth_core_with_no_contract_tx(
            &init_block.to_json().unwrap().to_string(),
            &chain_id,
            gas_price,
            confs as u64,
            state,
            is_native,
        )
        .unwrap();
        last_submitted_block_idx += 1;

        // Now, ALL the special hashes will be the init blocks hash.
        let mut hashes = db_utils.get_special_hashes();
        let mut expected_hashes = SpecialHashes {
            linker: genesis_hash,
            tail: block_hashes[0].clone(),
            canon: block_hashes[0].clone(),
            latest: block_hashes[0].clone(),
            anchor: block_hashes[0].clone(),
        };
        assert_eq!(hashes, expected_hashes);

        // Now add a block...
        append_to_blockchain(&db_utils, &blocks[last_submitted_block_idx], true).unwrap();
        last_submitted_block_idx += 1;

        hashes = db_utils.get_special_hashes();
        expected_hashes = SpecialHashes {
            linker: genesis_hash,
            tail: block_hashes[0].clone(),
            canon: block_hashes[0].clone(),
            latest: block_hashes[1].clone(),
            anchor: block_hashes[0].clone(),
        };
        assert_eq!(hashes, expected_hashes);

        // Now let's add enough blocks to pull the canon away from the tail...
        let end_idx = confs + 1;
        (last_submitted_block_idx..=end_idx).for_each(|i| {
            append_to_blockchain(&db_utils, &blocks[i], true).unwrap();
            last_submitted_block_idx = i;
        });
        hashes = db_utils.get_special_hashes();
        expected_hashes = SpecialHashes {
            linker: genesis_hash,
            tail: block_hashes[0].clone(),
            canon: block_hashes[last_submitted_block_idx - confs].clone(),
            latest: block_hashes[end_idx].clone(),
            anchor: block_hashes[0].clone(),
        };
        assert_eq!(hashes, expected_hashes);

        // And another block...
        append_to_blockchain(&db_utils, &blocks[last_submitted_block_idx + 1], true).unwrap();
        last_submitted_block_idx += 1;
        hashes = db_utils.get_special_hashes();
        expected_hashes = SpecialHashes {
            tail: block_hashes[0].clone(),
            canon: block_hashes[last_submitted_block_idx - confs].clone(),
            latest: block_hashes[last_submitted_block_idx].clone(),
            anchor: block_hashes[0].clone(),
            linker: EthHash::from_slice(&db_utils.get_eth_ptoken_genesis_hash_key()),
        };
        assert_eq!(hashes, expected_hashes);

        // Now let's check that we can't add a block that's _past_ the next one...
        match append_to_blockchain(&db_utils, &blocks[last_submitted_block_idx + 2], true) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::NoParentError(e)) => {
                let n = blocks[last_submitted_block_idx + 2]
                    .get_block_number()
                    .unwrap()
                    .as_u64();
                assert_eq!(e.block_num, n)
            },
            Err(e) => panic!("Wrong error received: {e}!"),
        }

        // Or that we can't add the same block again...
        match append_to_blockchain(&db_utils, &blocks[last_submitted_block_idx], true) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::BlockAlreadyInDbError(e)) => {
                let expected_err = BlockAlreadyInDbError::new(
                    blocks[last_submitted_block_idx].get_block_number().unwrap().as_u64(),
                    "✘ Block Rejected - it's already in the db!".to_string(),
                    BridgeSide::Native,
                );
                assert_eq!(e, expected_err)
            },
            Err(e) => panic!("Wrong error received: {e}!"),
        }

        // Or a previous one...
        match append_to_blockchain(&db_utils, &blocks[last_submitted_block_idx - 1], true) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::BlockAlreadyInDbError(e)) => {
                let expected_err = BlockAlreadyInDbError::new(
                    blocks[last_submitted_block_idx - 1]
                        .get_block_number()
                        .unwrap()
                        .as_u64(),
                    "✘ Block Rejected - it's already in the db!".to_string(),
                    BridgeSide::Native,
                );
                assert_eq!(e, expected_err)
            },
            Err(e) => panic!("Wrong error received: {e}!"),
        }

        // Just for safety incase the test version of the tail length changes...
        assert!(
            blocks.len() as u64 > ETH_TAIL_LENGTH,
            "ETH_TAIL_LENGTH too big for number of sample blocks!"
        );

        // Now let's add enough blocks to start moving the tail away from the anchor...
        (last_submitted_block_idx + 1..=(ETH_TAIL_LENGTH as usize + confs)).for_each(|i| {
            append_to_blockchain(&db_utils, &blocks[i], true).unwrap();
            last_submitted_block_idx = i;
        });
        hashes = db_utils.get_special_hashes();
        expected_hashes = SpecialHashes {
            linker: genesis_hash, // FIXME should have changed?
            tail: block_hashes[last_submitted_block_idx - confs - ETH_TAIL_LENGTH as usize].clone(),
            canon: block_hashes[last_submitted_block_idx - confs].clone(),
            latest: block_hashes[last_submitted_block_idx].clone(),
            anchor: block_hashes[0].clone(),
        };
        assert_eq!(hashes, expected_hashes);

        // Another block should start the linker hash changing...
        append_to_blockchain(&db_utils, &blocks[last_submitted_block_idx + 1], true).unwrap();
        last_submitted_block_idx += 1;
        hashes = db_utils.get_special_hashes();
        let mut expected_linker_hash = calculate_linker_hash(
            blocks[0].get_block_hash().unwrap(), // Block to link to...
            blocks[0].get_block_hash().unwrap(), // Anchor block hash...
            genesis_hash.clone(),                // Current linker hash
        );
        expected_hashes = SpecialHashes {
            linker: expected_linker_hash,
            tail: block_hashes[last_submitted_block_idx - confs - ETH_TAIL_LENGTH as usize].clone(),
            canon: block_hashes[last_submitted_block_idx - confs].clone(),
            latest: block_hashes[last_submitted_block_idx].clone(),
            anchor: block_hashes[0].clone(),
        };
        assert_eq!(hashes, expected_hashes);

        // And a final block should mean old blocks beyond the tail are now being removed (this
        // didn't happen with the previous submission because the one block older than the tail was
        // in fact the anchor block, which will never be removed.
        append_to_blockchain(&db_utils, &blocks[last_submitted_block_idx + 1], true).unwrap();
        last_submitted_block_idx += 1;
        hashes = db_utils.get_special_hashes();
        expected_linker_hash = calculate_linker_hash(
            blocks[1].get_block_hash().unwrap(), // Block to link to...
            blocks[0].get_block_hash().unwrap(), // Anchor block hash...
            expected_linker_hash,
        );
        expected_hashes = SpecialHashes {
            linker: expected_linker_hash,
            tail: block_hashes[last_submitted_block_idx - confs - ETH_TAIL_LENGTH as usize].clone(),
            canon: block_hashes[last_submitted_block_idx - confs].clone(),
            latest: block_hashes[last_submitted_block_idx].clone(),
            anchor: block_hashes[0].clone(),
        };
        assert_eq!(hashes, expected_hashes);

        // And finally, because we're now using the linker hash, we assert the block that's been
        // linked to has been removed because it's now between the tail and the anchor blocks.
        match db_utils.get_submission_material_from_db(&blocks[1].get_block_hash().unwrap()) {
            Ok(_) => panic!("This block should not be in the db!"),
            Err(AppError::Custom(e)) => {
                let expected_err = "Cannot find item in database!";
                assert_eq!(e, expected_err)
            },
            Err(e) => panic!("Wrong error received: {e}!"),
        }
    }
}
