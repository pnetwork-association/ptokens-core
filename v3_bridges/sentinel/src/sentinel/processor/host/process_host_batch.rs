use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{append_to_blockchain, EthSubmissionMaterial, EthSubmissionMaterials, HostDbUtils};
use ethereum_types::Address as EthAddress;
use lib::{HostOutput, SentinelDbUtils, SentinelError, UserOperations};

const SIDE: &str = "host";
const ORIGIN_NETWORK_ID: Vec<u8> = vec![]; // FIXME calculate this!

pub fn process_host<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    sub_mat: &EthSubmissionMaterial,
    state_manager: &EthAddress,
    is_validating: bool,
    use_db_tx: bool,
    dry_run: bool,
) -> Result<UserOperations, SentinelError> {
    if use_db_tx {
        debug!("Starting db tx in host processor!");
        db.start_transaction()?;
    }

    let n = sub_mat.get_block_number()?;
    let db_utils = HostDbUtils::new(db);

    if dry_run {
        warn!("Dry running so skipping block chain appending step!");
    } else {
        append_to_blockchain(&db_utils, sub_mat, is_validating)?;
    }

    if !is_in_sync {
        warn!("{SIDE} is not in sync, not processing receipts!");
        return Ok(UserOperations::empty());
    }

    if sub_mat.receipts.is_empty() {
        debug!("{SIDE} block {n} had no receipts to process!");
        return Ok(UserOperations::empty());
    }

    let r = if is_validating {
        sub_mat.receipts_are_valid()?;
        UserOperations::from_sub_mat(BridgeSide::Host, &sub_mat, state_manager, &ORIGIN_NETWORK_ID)?
    } else {
        UserOperations::empty()
    };

    debug!("Finished processing {SIDE} block {n}!");

    if use_db_tx {
        debug!("Ending db tx in host processor!");
        db.end_transaction()?;
    }
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
    let use_db_tx = false;
    let dry_run = false;

    let user_ops = UserOperations::from(
        batch
            .iter()
            .map(|sub_mat| {
                process_host(
                    db,
                    is_in_sync,
                    sub_mat,
                    state_manager,
                    is_validating,
                    use_db_tx,
                    dry_run,
                )
            })
            .collect::<Result<Vec<UserOperations>, SentinelError>>()?,
    );

    let mut output = HostOutput::new(batch.get_last_block_num()?)?;

    if !user_ops.is_empty() {
        let db_utils = SentinelDbUtils::new(db);

        let mut host_user_ops = db_utils.get_host_user_operations()?;
        let native_user_ops = db_utils.get_native_user_operations()?;
        host_user_ops.add(user_ops);

        let (native, host) = native_user_ops.remove_matches(host_user_ops);
        output.add_unmatched_user_ops(&native, &host);

        // TODO need to send native and host to mongo since these are our currently unmatched user ops!
        db_utils.add_native_user_operations(native)?;
        db_utils.add_host_user_operations(host)?;
    }

    db.end_transaction()?;

    info!("Finished processing {SIDE} submission material!");
    Ok(output)
}
