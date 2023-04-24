use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{append_to_blockchain, EthSubmissionMaterial, EthSubmissionMaterials, HostDbUtils, NativeDbUtils};
use ethereum_types::Address as EthAddress;
use lib::{Output, SentinelDbUtils, SentinelError, UserOpList, UserOps};

lazy_static::lazy_static! {
    static ref ORIGIN_NETWORK_ID: Vec<u8> = hex::decode("e15503e4").unwrap();// FIXME calculate this!
}

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
) -> Result<UserOps, SentinelError> {
    if use_db_tx {
        debug!("Starting db tx in {side} processor!");
        db.start_transaction()?;
    }

    let n = sub_mat.get_block_number()?;

    if dry_run {
        warn!("Dry running so skipping block chain appending step!");
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
        sub_mat.receipts_are_valid()?;
    };

    let r = UserOps::from_sub_mat(side, state_manager, &ORIGIN_NETWORK_ID, router, sub_mat)?;

    if use_db_tx {
        debug!("ending db tx in {side} processor!");
        db.end_transaction()?;
    }

    debug!("finished processing {side} block {n} - user ops: {r}");
    Ok(r)
}

pub fn process_batch<D: DatabaseInterface>(
    db: &D,
    router: &EthAddress,
    state_manager: &EthAddress,
    batch: &EthSubmissionMaterials,
    is_validating: bool,
    side: BridgeSide,
) -> Result<Output, SentinelError> {
    info!("Processing {side} batch of submission material...");
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
                )
            })
            .collect::<Result<Vec<UserOps>, SentinelError>>()?,
    );

    let ops_requiring_txs = UserOpList::process_ops(&SentinelDbUtils::new(db), user_ops)?;

    let output = Output::new(side, batch.get_last_block_num()?, ops_requiring_txs);

    db.end_transaction()?;

    info!("finished processing {side} submission material");

    output
}
