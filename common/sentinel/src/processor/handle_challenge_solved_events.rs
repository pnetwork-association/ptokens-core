use std::result::Result;

use common::DatabaseInterface;
use common_eth::EthSubmissionMaterial;
use ethereum_types::Address as EthAddress;

use crate::{ChallengeSolvedEvents, ChallengesList, SentinelDbUtils, SentinelError};

pub(super) fn maybe_handle_challenge_solved_events<D: DatabaseInterface>(
    db_utils: &SentinelDbUtils<D>,
    pnetwork_hub: &EthAddress,
    sub_mat: &EthSubmissionMaterial,
    sentinel_address: &EthAddress,
) -> Result<(), SentinelError> {
    if sub_mat.receipts.is_empty() {
        debug!("no receipts in sub mat so not checking for new challenges");
        return Ok(());
    }

    // FIXME needs filtering by actor address etc
    let ids = ChallengeSolvedEvents::from_sub_mat(sub_mat, pnetwork_hub, sentinel_address)?.to_ids()?;

    if ids.is_empty() {
        debug!("no solved challenges found in sub mat");
        return Ok(());
    }

    let mut list = ChallengesList::get(db_utils);
    list.update_challenge_statuses_to_solved(db_utils, ids)?;

    Ok(())
}
