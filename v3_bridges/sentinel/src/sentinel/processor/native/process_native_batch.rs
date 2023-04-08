use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{append_to_blockchain, EthSubmissionMaterial, EthSubmissionMaterials, NativeDbUtils};
use ethereum_types::Address as EthAddress;
use lib::{NativeOutput, SentinelDbUtils, SentinelError, UserOpList, UserOps};

const SIDE: &str = "native";
const ORIGIN_NETWORK_ID: Vec<u8> = vec![]; // FIXME calculate this!

pub fn process_native<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    sub_mat: &EthSubmissionMaterial,
    state_manager: &EthAddress,
    is_validating: bool,
    use_db_tx: bool,
    dry_run: bool,
) -> Result<UserOps, SentinelError> {
    if use_db_tx {
        debug!("Starting db tx in host processor!");
        db.start_transaction()?;
    }

    let n = sub_mat.get_block_number()?;
    let db_utils = NativeDbUtils::new(db);

    if dry_run {
        warn!("Dry running so skipping block chain appending step!");
    } else {
        append_to_blockchain(&db_utils, sub_mat, is_validating)?;
    }

    if !is_in_sync {
        warn!("{SIDE} is not in sync, not processing receipts!");
        return Ok(UserOps::empty());
    }

    if sub_mat.receipts.is_empty() {
        debug!("Native block {n} had no receipts to process!");
        return Ok(UserOps::empty());
    }

    let r = if is_validating {
        sub_mat.receipts_are_valid()?;
        UserOps::from_sub_mat(BridgeSide::Native, sub_mat, state_manager, &ORIGIN_NETWORK_ID)?
    } else {
        UserOps::empty()
    };

    if use_db_tx {
        debug!("Ending db tx in host processor!");
        db.end_transaction()?;
    }

    debug!("Finished processing {SIDE} block {n}!");
    Ok(r)
}

pub fn process_native_batch<D: DatabaseInterface>(
    db: &D,
    is_in_sync: bool,
    state_manager: &EthAddress,
    batch: &EthSubmissionMaterials,
    is_validating: bool,
) -> Result<NativeOutput, SentinelError> {
    info!("Processing {SIDE} batch of submission material...");
    db.start_transaction()?;
    let use_db_tx = false;
    let dry_run = false;

    let user_ops = UserOps::from(
        batch
            .iter()
            .map(|sub_mat| {
                process_native(
                    db,
                    is_in_sync,
                    sub_mat,
                    state_manager,
                    is_validating,
                    use_db_tx,
                    dry_run,
                )
            })
            .collect::<Result<Vec<UserOps>, SentinelError>>()?,
    );

    let _ops_requiring_txs = UserOpList::process_ops(&SentinelDbUtils::new(db), user_ops)?;

    let output = NativeOutput::new(batch.get_last_block_num()?)?;

    db.end_transaction()?;

    info!("Finished processing {SIDE} submission material!");
    Ok(output)
}
