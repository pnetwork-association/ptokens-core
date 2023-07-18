use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{append_to_blockchain, EthSubmissionMaterial, EthSubmissionMaterials, HostDbUtils, NativeDbUtils};
use ethereum_types::Address as EthAddress;

use crate::{Bytes4, ProcessorOutput, SentinelDbUtils, SentinelError, UserOpList, UserOps};

#[allow(clippy::too_many_arguments)]
pub fn process_single<D: DatabaseInterface>(
    db: &D,
    sub_mat: &EthSubmissionMaterial,
    pnetwork_hub: &EthAddress,
    is_validating: bool,
    use_db_tx: bool,
    dry_run: bool,
    side: BridgeSide,
    network_id: &Bytes4,
    reprocess: bool,
) -> Result<UserOps, SentinelError> {
    if use_db_tx {
        debug!("Starting db tx in {side} processor!");
        db.start_transaction()?;
    }

    let n = sub_mat.get_block_number()?;

    if dry_run {
        warn!("dry running so skipping block chain appending step");
    } else if reprocess {
        warn!("reprocessing so skipping block chain appending step");
    } else if side.is_native() {
        append_to_blockchain(&NativeDbUtils::new(db), sub_mat, is_validating)?;
    } else {
        append_to_blockchain(&HostDbUtils::new(db), sub_mat, is_validating)?;
    }

    if sub_mat.receipts.is_empty() {
        debug!("{side} block {n} had no receipts to process!");
        return Ok(UserOps::empty());
    }

    if is_validating {
        // NOTE: Block header gets validated above when appending to the chain.
        sub_mat.receipts_are_valid()?;
    };

    let ops = UserOps::from_sub_mat(side, &network_id.to_vec(), pnetwork_hub, sub_mat)?;
    debug!("found user ops: {ops}");

    let sentinel_db_utils = SentinelDbUtils::new(db);
    let mut user_op_list = UserOpList::get(&sentinel_db_utils);
    user_op_list.process_ops(ops.clone(), &sentinel_db_utils)?;

    if use_db_tx {
        debug!("ending db tx in {side} processor!");
        db.end_transaction()?;
    }
    debug!("finished processing {side} block {n}");

    Ok(ops)
}

#[allow(clippy::too_many_arguments)]
pub fn process_batch<D: DatabaseInterface>(
    db: &D,
    pnetwork_hub: &EthAddress,
    batch: &EthSubmissionMaterials,
    is_validating: bool,
    side: BridgeSide,
    network_id: &Bytes4,
    reprocess: bool,
) -> Result<ProcessorOutput, SentinelError> {
    info!("Processing {side} batch of submission material...");
    let use_db_tx = false;
    let dry_run = false;

    let processed_user_ops = UserOps::from(
        batch
            .iter()
            .map(|sub_mat| {
                process_single(
                    db,
                    sub_mat,
                    pnetwork_hub,
                    is_validating,
                    use_db_tx,
                    dry_run,
                    side,
                    network_id,
                    reprocess,
                )
            })
            .collect::<Result<Vec<UserOps>, SentinelError>>()?,
    );

    info!("finished processing {side} submission material");
    let r = ProcessorOutput::new(side, batch.get_last_block_num()?, processed_user_ops)?;
    Ok(r)
}
