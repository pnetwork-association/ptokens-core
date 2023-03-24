use std::result::Result;

use common::DatabaseInterface;
use common_eth::{append_to_blockchain, EthSubmissionMaterial, EthSubmissionMaterials, NativeDbUtils};
use lib::{NativeAddressesAndTopics, NativeOutput, RelevantLogsFromBlock, SentinelError};

const SIDE: &str = "native";
/*
Okay so we need relevant logs in three flavours:

- logs we expect to see (on the host chain)
- logs we expected to se (because of the host chain)
- logs we need to query (if we're in sync we can send these straight to a broadcaster, otherwise save them in core db)

We also need to save stuff in epochs but for now we can save it in two mega structs instead, one each for host and native.
 */

fn process_native<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    sub_mat: &EthSubmissionMaterial,
    addresses_and_topics: &NativeAddressesAndTopics,
    is_validating: bool,
) -> Result<RelevantLogsFromBlock, SentinelError> {
    let n = sub_mat.get_block_number()?;
    let db_utils = NativeDbUtils::new(db);
    append_to_blockchain(&db_utils, sub_mat, is_validating)?;
    let empty_logs = RelevantLogsFromBlock::default();

    if !is_in_sync {
        warn!("{SIDE} is not in sync, not processing receipts!");
        return Ok(empty_logs);
    }

    if sub_mat.receipts.is_empty() {
        debug!("Native block {n} had no receipts to process!");
        return Ok(empty_logs);
    }

    if is_validating {
        sub_mat.receipts_are_valid()?;
    }

    debug!("Finished processing {SIDE} block {n}!");
    Ok(RelevantLogsFromBlock::from_eth_receipts(
        sub_mat.get_block_number()?.as_u64(),
        sub_mat.get_timestamp(),
        &sub_mat.receipts,
        &**addresses_and_topics,
    ))
}

pub fn process_native_batch<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    batch: &EthSubmissionMaterials,
    addresses_and_topics: &NativeAddressesAndTopics,
    is_validating: bool,
) -> Result<NativeOutput, SentinelError> {
    info!("Processing {SIDE} batch of submission material...");

    db.start_transaction()?;
    let result = batch
        .iter()
        .map(|m| process_native(db, is_in_sync, m, addresses_and_topics, is_validating))
        .collect::<Result<Vec<RelevantLogsFromBlock>, SentinelError>>();
    db.end_transaction()?;

    match result {
        Err(e) => Err(e),
        Ok(_relevant_logs) => {
            info!("Finished processing {SIDE} submission material!");
            NativeOutput::new(batch.get_last_block_num()?)
        },
    }
}
