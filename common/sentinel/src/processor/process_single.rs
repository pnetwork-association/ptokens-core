use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{Chain, ChainDbUtils, EthSubmissionMaterial};
use ethereum_types::Address as EthAddress;

use crate::{NetworkId, SentinelDbUtils, SentinelError, UserOpList, UserOps};

pub(super) fn process_single<D: DatabaseInterface>(
    db: &D,
    sub_mat: EthSubmissionMaterial,
    pnetwork_hub: &EthAddress,
    validate: bool,
    _use_db_tx: bool,
    dry_run: bool,
    side: BridgeSide,
    network_id: &NetworkId,
    reprocess: bool,
    chain: &mut Chain,
) -> Result<UserOps, SentinelError> {
    let mcid = *chain.chain_id();
    // FIXE All db tx stuff currently comment out due to the below msg
    /* // FIXME These are handled in the strongbox core, and this breaks that. Think of a way to
     * get dry run capabilities back
    if use_db_tx {
        debug!("Starting db tx in {mcid} processor!");
        db.start_transaction()?;
    }
    */

    let chain_db_utils = ChainDbUtils::new(db);
    let n = sub_mat.get_block_number()?;

    if dry_run {
        warn!("dry running so skipping block chain appending step");
    } else if reprocess {
        warn!("reprocessing so skipping block chain appending step");
    } else {
        chain.insert(&chain_db_utils, sub_mat, validate)?;
    };

    let maybe_canon_block = chain.get_canonical_sub_mat(&chain_db_utils)?;
    if maybe_canon_block.is_none() {
        warn!("there is no canonical block on chain {mcid} yet");
        /*
        if use_db_tx {
            db.end_transaction()?;
        };
        */
        return Ok(UserOps::empty());
    }

    let canonical_sub_mat = maybe_canon_block.expect("this not to fail due to above check");
    if canonical_sub_mat.receipts.is_empty() {
        debug!("{mcid} canon block had no receipts to process");
        /*
        if use_db_tx {
            db.end_transaction()?;
        };
        */
        return Ok(UserOps::empty());
    }

    let ops = UserOps::from_sub_mat(side, network_id, pnetwork_hub, &canonical_sub_mat)?;
    debug!("found user ops: {ops}");

    let sentinel_db_utils = SentinelDbUtils::new(db);
    let mut user_op_list = UserOpList::get(&sentinel_db_utils);
    user_op_list.process_ops(ops.clone(), &sentinel_db_utils)?;

    /*
    if use_db_tx {
        debug!("ending db tx in {mcid} processor!");
        db.end_transaction()?;
    }
    */
    debug!("finished processing {mcid} block {n}");

    Ok(ops)
}
