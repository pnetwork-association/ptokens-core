use std::result::Result;

use common::DatabaseInterface;
use common_eth::{append_to_blockchain, EthDbUtilsExt, EthSubmissionMaterial, EthSubmissionMaterials, NativeDbUtils};
use lib::{NativeOutput, SentinelError};

const SIDE: &str = "native";

fn process_native<D: DatabaseInterface>(db: &D, sub_mat: &EthSubmissionMaterial) -> Result<(), SentinelError> {
    let n = sub_mat.get_block_number()?;
    let db_utils = NativeDbUtils::new(db);
    append_to_blockchain(&db_utils, sub_mat)?;

    if sub_mat.receipts.is_empty() {
        debug!("Native block {n} had no receipts to process!");
        Ok(())
    } else {
        debug!("Finished processing {SIDE} block {n}!");
        Ok(())
    }
}

pub fn process_native_batch<D: DatabaseInterface>(
    db: &D,
    batch: &EthSubmissionMaterials,
) -> Result<NativeOutput, SentinelError> {
    info!("Processing {SIDE} batch of submission material...");

    db.start_transaction()?;
    let result = batch
        .iter()
        .map(|m| process_native(db, m))
        .collect::<Result<Vec<()>, SentinelError>>();
    db.end_transaction()?;

    match result {
        Ok(_) => {
            info!("Finished processing {SIDE} submission material!");
            NativeOutput::new(batch.get_last_block_num()?)
        },
        Err(SentinelError::NoParent(_)) => {
            let n = NativeDbUtils::new(db).get_latest_eth_block_number()? + 1;
            warn!("No parent error in {SIDE} proocessor - need to restart from {n}!");
            Err(SentinelError::SyncerRestart(n as u64))
        },
        Err(e) => Err(e),
    }
}
