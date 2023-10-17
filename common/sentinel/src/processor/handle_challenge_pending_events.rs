use std::result::Result;

use common::DatabaseInterface;
use common_eth::EthSubmissionMaterial;
use ethereum_types::Address as EthAddress;

use crate::{Challenges, ChallengesList, SentinelDbUtils, SentinelError};

pub(super) fn maybe_handle_challenge_pending_events<D: DatabaseInterface>(
    db_utils: &SentinelDbUtils<D>,
    pnetwork_hub: &EthAddress,
    sub_mat: &EthSubmissionMaterial,
) -> Result<(), SentinelError> {
    if sub_mat.receipts.is_empty() {
        debug!("no receipts in sub mat so not checking for new challenges");
        return Ok(());
    }

    let challenges = Challenges::from_sub_mat(sub_mat, pnetwork_hub)?;
    if challenges.is_empty() {
        debug!("no challenges found in sub mat");
        return Ok(());
    }

    let list = ChallengesList::get(db_utils);
    list.add_challenges(db_utils, challenges)?;
    Ok(())
}
