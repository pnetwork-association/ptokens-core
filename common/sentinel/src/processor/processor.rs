use std::result::Result;

use common::{BridgeSide, DatabaseInterface};
use common_eth::{Chain, ChainDbUtils, EthSubmissionMaterial, EthSubmissionMaterials};
use common_metadata::MetadataChainId;
use ethereum_types::Address as EthAddress;

use crate::{Bytes4, ProcessorOutput, SentinelDbUtils, SentinelError, UserOpList, UserOps};

pub fn process_single<D: DatabaseInterface>(
    db: &D,
    sub_mat: EthSubmissionMaterial,
    pnetwork_hub: &EthAddress,
    validate: bool,
    _use_db_tx: bool,
    dry_run: bool,
    side: BridgeSide,
    network_id: &Bytes4,
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

    let ops = UserOps::from_sub_mat(side, &network_id.to_vec(), pnetwork_hub, &canonical_sub_mat)?;
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

pub fn process_batch<D: DatabaseInterface>(
    db: &D,
    pnetwork_hub: &EthAddress,
    batch: &EthSubmissionMaterials,
    validate: bool,
    side: BridgeSide,
    network_id: &Bytes4,
    reprocess: bool,
    dry_run: bool,
    mcid: MetadataChainId,
    governance_address: Option<EthAddress>
) -> Result<ProcessorOutput, SentinelError> {
    info!("Processing {mcid} batch of submission material...");

    let chain_db_utils = ChainDbUtils::new(db);
    let mut chain = Chain::get(&chain_db_utils, mcid)?;

    let use_db_tx = !dry_run;

    let processed_user_ops = UserOps::from(
        batch
            .iter()
            .map(|sub_mat| {
                process_single(
                    db,
                    sub_mat.clone(),
                    pnetwork_hub,
                    validate,
                    use_db_tx,
                    dry_run,
                    side,
                    network_id,
                    reprocess,
                    &mut chain,
                )
            })
            .collect::<Result<Vec<UserOps>, SentinelError>>()?,
    );

    info!("finished processing {side} submission material");
    let r = ProcessorOutput::new(side, batch.get_last_block_num()?, processed_user_ops)?;
    Ok(r)
}
