use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{append_to_blockchain, EthSubmissionMaterial, EthSubmissionMaterials, HostDbUtils, NativeDbUtils};
use ethereum_types::Address as EthAddress;
use lib::{Bytes4, DbUtilsT, Output, SentinelDbUtils, SentinelError, UserOpList, UserOps};

#[allow(clippy::too_many_arguments)]
pub fn process_single<D: DatabaseInterface>(
    db: &D,
    router: &EthAddress,
    sub_mat: &EthSubmissionMaterial,
    state_manager: &EthAddress,
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

    let r = UserOps::from_sub_mat(side, state_manager, &network_id.to_vec(), router, sub_mat)?;

    if use_db_tx {
        debug!("ending db tx in {side} processor!");
        db.end_transaction()?;
    }

    debug!("finished processing {side} block {n} - user ops: {r}");
    Ok(r)
}

#[allow(clippy::too_many_arguments)]
pub fn process_batch<D: DatabaseInterface>(
    db: &D,
    router: &EthAddress,
    state_manager: &EthAddress,
    batch: &EthSubmissionMaterials,
    is_validating: bool,
    side: BridgeSide,
    network_id: &Bytes4,
    reprocess: bool,
) -> Result<Output, SentinelError> {
    info!("Processing {side} batch of submission material...");
    // FIXME db transaction handling - make sure it works for dry runs etc

    db.start_transaction()?;

    let use_db_tx = false;
    let dry_run = false;

    let user_ops = UserOps::from(
        batch
            .iter()
            .map(|sub_mat| {
                process_single(
                    db,
                    router,
                    sub_mat,
                    state_manager,
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

    // FIXME db transaction/dry run stuff
    let db_utils = SentinelDbUtils::new(db);
    let mut user_op_list = UserOpList::get(&db_utils);
    user_op_list.process_ops(user_ops, &db_utils)?;

    // TODO: Get cancellable ops via last X ops, check for cancellable, return those.
    let cancellable_ops = UserOps::default();
    // Then, using the same list, get the last X ops from it. (TODO fix flags so that we can be more efficient here)
    // Then parse last X ops to see if any are cancellable. This meansjbhb

    db.end_transaction()?;

    let output = Output::new(side, batch.get_last_block_num()?, cancellable_ops);

    info!("finished processing {side} submission material");

    output
}
/*
plan:
get last X user ops from list after processing the batch.
go through them getting any that are cancellable
add news checks re time stuff for cancellability
return what's cancellable

sub_mat.get_timestamp()
or
sub_mats.get_last_block_timestamp()
then use that.
*/
