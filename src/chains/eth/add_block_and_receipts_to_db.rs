use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::{
        eth_block_and_receipts::EthBlockAndReceipts,
        eth_database_utils::{
            eth_block_exists_in_db,
            put_eth_block_and_receipts_in_db,
        },
    },
};

pub fn add_block_and_receipts_to_db_if_not_extant<D>(
    db: &D,
    block_and_receipts: &EthBlockAndReceipts,
) -> Result<()>
    where D: DatabaseInterface
{
    info!("✔ Adding ETH block and receipts if not already in db...");
    match eth_block_exists_in_db(db, &block_and_receipts.block.hash) {
        false => {
            info!("✔ Block & receipts not in db, adding them now...");
            put_eth_block_and_receipts_in_db(db, block_and_receipts)
        }
        true => Err("✘ Block Rejected - it's already in the db!".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_utils::get_test_database,
        btc_on_eth::eth::eth_test_utils::get_sample_eth_block_and_receipts_n,
    };


    #[test]
    fn should_maybe_add_block_and_receipts_to_db() {
        let db = get_test_database();
        let block_and_receipts = get_sample_eth_block_and_receipts_n(1).unwrap();
        let eth_block_hash = block_and_receipts.block.hash;
        let bool_before = eth_block_exists_in_db(&db, &eth_block_hash);
        assert!(!bool_before);
        if let Err(e) = add_block_and_receipts_to_db_if_not_extant(&db, &block_and_receipts) {
            panic!("Error when maybe adding block to database: {}", e);
        }
        let bool_after = eth_block_exists_in_db(&db, &eth_block_hash);
        assert!(bool_after);
    }

    #[test]
    fn should_error_if_block_already_in_db() {
        let db = get_test_database();
        let block_and_receipts = get_sample_eth_block_and_receipts_n(1).unwrap();
        let eth_block_hash = block_and_receipts.block.hash;
        let bool_before = eth_block_exists_in_db(&db, &eth_block_hash);
        assert!(!bool_before);
        if let Err(e) = add_block_and_receipts_to_db_if_not_extant(&db, &block_and_receipts) {
            panic!("Error when maybe adding block to database: {}", e);
        };
        let bool_after = eth_block_exists_in_db(&db, &eth_block_hash);
        if add_block_and_receipts_to_db_if_not_extant(&db, &block_and_receipts).is_ok() {
            panic!("Should error ∵ block already in db!");
        }
        assert!(bool_after);
    }
}
