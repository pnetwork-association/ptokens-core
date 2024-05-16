use std::result::Result;

use common::DatabaseInterface;
use common_eth::{Chain, ChainDbUtils, EthSubmissionMaterial};

use crate::{NetworkConfig, SentinelError, SignedEvents};

pub(super) fn process_single<D: DatabaseInterface>(
    db: &D,
    sub_mat: EthSubmissionMaterial,
    validate: bool,
    dry_run: bool,
    network_config: &NetworkConfig,
    reprocess: bool,
    chain: &mut Chain,
) -> Result<SignedEvents, SentinelError> {
    let mcid = *chain.chain_id();
    // NOTE: All db transaction stuff is handled via strongbox

    let chain_db_utils = ChainDbUtils::new(db);
    let n = sub_mat.get_block_number()?;

    let mut maybe_canon_block = None;

    if dry_run {
        warn!("dry running so skipping block chain appending step");
    } else if reprocess {
        warn!("reprocessing so skipping block chain appending step");
        // NOTE: This is to avoid having to clone the submat in below arm, since reprocessing is much
        // rarer
        maybe_canon_block = Some(sub_mat);
    } else {
        chain.insert(&chain_db_utils, sub_mat, validate)?;
    };

    if !reprocess {
        maybe_canon_block = chain.get_canonical_sub_mat(&chain_db_utils)?
    };

    if maybe_canon_block.is_none() {
        warn!("there is no canonical block on chain {mcid} yet");
        return Ok(SignedEvents::empty());
    }

    let canonical_sub_mat = maybe_canon_block.expect("this not to fail due to above check");
    if canonical_sub_mat.receipts.is_empty() {
        debug!("{mcid} canon block had no receipts to process");
        return Ok(SignedEvents::empty());
    }

    let signed_events = SignedEvents::try_from((
        &chain.mcid(),
        &chain_db_utils.get_pk()?,
        &canonical_sub_mat,
        network_config,
    ))?;

    debug!("found signed events: {signed_events:?}");
    debug!("finished processing {mcid} block {n}");

    Ok(signed_events)
}
