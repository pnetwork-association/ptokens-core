use std::result::Result;

use common::DatabaseInterface;
use common_eth::{append_to_blockchain, EthSubmissionMaterial, EthSubmissionMaterials, HostDbUtils};
use ethereum_types::Address as EthAddress;
use lib::{HostOutput, RelevantLogs, RelevantLogsFromBlock, SentinelDbUtils, SentinelError};

const SIDE: &str = "host";

fn process_host<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    sub_mat: &EthSubmissionMaterial,
    state_manager: &EthAddress,
    is_validating: bool,
) -> Result<RelevantLogsFromBlock, SentinelError> {
    let n = sub_mat.get_block_number()?;
    let db_utils = HostDbUtils::new(db);
    append_to_blockchain(&db_utils, sub_mat, is_validating)?;
    let empty_logs = RelevantLogsFromBlock::default();

    if !is_in_sync {
        warn!("{SIDE} is not in sync, not processing receipts!");
        return Ok(empty_logs);
    }

    if sub_mat.receipts.is_empty() {
        debug!("{SIDE} block {n} had no receipts to process!");
        return Ok(empty_logs);
    }

    let r = if is_validating {
        sub_mat.receipts_are_valid()?;
        RelevantLogsFromBlock::from_eth_receipts(
            sub_mat.get_block_number()?.as_u64(),
            sub_mat.get_timestamp(),
            &sub_mat.receipts,
            state_manager,
        )
    } else {
        RelevantLogsFromBlock::empty()
    };

    debug!("Finished processing {SIDE} block {n}!");
    Ok(r)
}

pub fn process_host_batch<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    state_manager: &EthAddress,
    batch: &EthSubmissionMaterials,
    is_validating: bool,
) -> Result<HostOutput, SentinelError> {
    info!("Processing {SIDE} batch of submission material...");
    db.start_transaction()?;

    let relevant_logs = RelevantLogs::new(
        batch
            .iter()
            .map(|sub_mat| process_host(db, is_in_sync, sub_mat, state_manager, is_validating))
            .collect::<Result<Vec<RelevantLogsFromBlock>, SentinelError>>()?,
    );

    if !relevant_logs.is_empty() {
        SentinelDbUtils::new(db).add_host_relevant_logs(relevant_logs)?;
    }

    db.end_transaction()?;

    info!("Finished processing {SIDE} submission material!");
    HostOutput::new(batch.get_last_block_num()?)
}
