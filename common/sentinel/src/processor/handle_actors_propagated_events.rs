use std::result::Result;

use common::DatabaseInterface;
use common_eth::EthSubmissionMaterial;
use ethereum_types::Address as EthAddress;

use crate::{Actor, Actors, NetworkId, SentinelDbUtils, SentinelError};

pub(super) fn maybe_handle_actors_propagated_events<D: DatabaseInterface>(
    db_utils: &SentinelDbUtils<D>,
    network_id: &NetworkId,
    governance_address: &EthAddress,
    sentinel_address: &EthAddress,
    sub_mat: &EthSubmissionMaterial,
) -> Result<(), SentinelError> {
    if sub_mat.receipts.is_empty() {
        debug!("no receipts in sub mat so not checking for new actors");
        return Ok(());
    }

    match Actors::from_sub_mat(sub_mat, *governance_address, *network_id)? {
        None => Ok(()),
        Some(actors) => match actors.get_inclusion_proof_for_actor(&Actor::from(sentinel_address)) {
            Err(e) => {
                error!("{e}");
                warn!("failed to create proof for this sentinel amongst actors!");
                // NOTE: We failed to get a proof for some reason - likely this sentinel not being
                // amongst the actors found in the event. We show the error in the logs here, but
                // continue as normal since the sentinel may not yet be registered, but may want to
                // continue syncing as usual.
                Ok(())
            },
            Ok(proof) => {
                // NOTE: We found a proof. Let's update in the db. (The update function first
                // checks that the proof for an epoch later than whatever one is currently stored
                // [if any] in the db).
                info!("successfully created sentinel inclusion proof: {proof}");
                proof.update_proof_in_db(db_utils)?;
                Ok(())
            },
        },
    }
}
