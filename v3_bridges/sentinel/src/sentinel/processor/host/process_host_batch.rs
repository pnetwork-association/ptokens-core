use std::result::Result;

use common::DatabaseInterface;
use common_eth::{append_to_blockchain, EthSubmissionMaterial, EthSubmissionMaterials, HostDbUtils};
use lib::{AddressesAndTopics, HostOutput, RelevantLogs, SentinelError};

const SIDE: &str = "host";

fn process_host<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    sub_mat: &EthSubmissionMaterial,
    addresses_and_topics: &AddressesAndTopics,
    is_validating: bool,
) -> Result<RelevantLogs, SentinelError> {
    let n = sub_mat.get_block_number()?;
    let db_utils = HostDbUtils::new(db);
    append_to_blockchain(&db_utils, sub_mat, is_validating)?;
    let empty_logs = RelevantLogs::default();

    if !is_in_sync {
        warn!("{SIDE} is not in sync, not processing receipts!");
        return Ok(empty_logs);
    }

    if sub_mat.receipts.is_empty() {
        debug!("{SIDE} block {n} had no receipts to process!");
        return Ok(empty_logs);
    }

    if is_validating {
        sub_mat.receipts_are_valid()?;
    }

    debug!("Finished processing {SIDE} block {n}!");
    Ok(RelevantLogs::from_eth_receipts(
        &addresses_and_topics,
        &sub_mat.receipts,
    ))
}

pub fn process_host_batch<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    batch: &EthSubmissionMaterials,
    addresses_and_topics: &AddressesAndTopics,
    is_validating: bool,
) -> Result<HostOutput, SentinelError> {
    info!("Processing {SIDE} batch of submission material...");
    db.start_transaction()?;
    let result = batch
        .iter()
        .map(|m| process_host(db, is_in_sync, m, addresses_and_topics, is_validating))
        .collect::<Result<Vec<RelevantLogs>, SentinelError>>();
    db.end_transaction()?;

    match result {
        Err(e) => Err(e),
        Ok(_relevant_logs) => {
            info!("Finished processing {SIDE} submission material!");
            HostOutput::new(batch.get_last_block_num()?)
        },
    }
}
