use std::result::Result;

use common::DatabaseInterface;
use common_eth::{append_to_blockchain, EthDbUtilsExt, EthSubmissionMaterial, EthSubmissionMaterials, HostDbUtils};
use lib::{HostOutput, SentinelError};

const SIDE: &str = "host";

fn process_host<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    sub_mat: &EthSubmissionMaterial,
) -> Result<(), SentinelError> {
    let n = sub_mat.get_block_number()?;
    let db_utils = HostDbUtils::new(db);
    append_to_blockchain(&db_utils, sub_mat)?;

    if !is_in_sync {
        warn!("{SIDE} is not in sync, not processing receipts!");
        Ok(())
    } else if sub_mat.receipts.is_empty() {
        debug!("{SIDE} block {n} had no receipts to process!");
        Ok(())
    } else {
        debug!("Finished processing {SIDE} block {n}!");
        Ok(())
    }
}

pub fn process_host_batch<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    batch: &EthSubmissionMaterials,
) -> Result<HostOutput, SentinelError> {
    info!("Processing {SIDE} batch of submission material...");
    db.start_transaction()?;
    let result = batch
        .iter()
        .map(|m| process_host(db, is_in_sync, m))
        .collect::<Result<Vec<()>, SentinelError>>();
    db.end_transaction()?;

    match result {
        Ok(_) => {
            info!("Finished processing {SIDE} submission material!");
            HostOutput::new(batch.get_last_block_num()?)
        },
        Err(SentinelError::NoParent(_)) => {
            let db_utils = HostDbUtils::new(db);
            let n = db_utils.get_latest_eth_block_number()? + 1;
            warn!("no parent error in {SIDE} proocessor - need to restart from {n}!");
            Err(SentinelError::SyncerRestart(n as u64))
        },
        Err(e) => Err(e),
    }
}
