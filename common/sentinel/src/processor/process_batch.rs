use std::result::Result;

use common::DatabaseInterface;
use common_eth::{Chain, ChainDbUtils, EthSubmissionMaterials};
use ethereum_types::Address as EthAddress;

use super::{
    maybe_handle_actors_propagated_events,
    maybe_handle_challenge_pending_events,
    maybe_handle_challenge_solved_events,
    process_single,
};
use crate::{NetworkConfig, ProcessorOutput, SentinelDbUtils, SentinelError, SignedEvents};

pub fn process_batch<D: DatabaseInterface>(
    db: &D,
    pnetwork_hub: &EthAddress,
    batch: &EthSubmissionMaterials,
    validate: bool,
    network_config: &NetworkConfig,
    reprocess: bool,
    dry_run: bool,
    maybe_governance_address: Option<EthAddress>,
    sentinel_address: EthAddress,
) -> Result<ProcessorOutput, SentinelError> {
    let network_id = network_config.network_id();
    info!("processing {network_id} batch of submission material...");

    let c_db_utils = ChainDbUtils::new(db);
    let s_db_utils = SentinelDbUtils::new(db);

    let mut chain = Chain::get(&c_db_utils, network_id.try_into()?)?;

    if let Some(ref governance_address) = maybe_governance_address {
        debug!("checking for events from governance address {governance_address}");
        // NOTE: If we find a governance address, it means we're on the governance chain, meaning
        // we need to watch out for `ActorsPropagated` events which are fired after epoch changes.
        // This changes this sentinel's `ActorInclusionProof` which is required to successfully
        // cancel a `UserOp`.
        batch.iter().try_for_each(|sub_mat| {
            maybe_handle_actors_propagated_events(
                &SentinelDbUtils::new(db),
                &network_id,
                governance_address,
                &sentinel_address,
                sub_mat,
            )
        })?
    };

    batch
        .iter()
        .try_for_each(|m| maybe_handle_challenge_pending_events(&s_db_utils, pnetwork_hub, m, &sentinel_address))?;

    batch
        .iter()
        .try_for_each(|m| maybe_handle_challenge_solved_events(&s_db_utils, pnetwork_hub, m, &sentinel_address))?;

    let signed_events = SignedEvents::from(
        batch
            .iter()
            .map(|sub_mat| {
                process_single(
                    db,
                    sub_mat.clone(),
                    validate,
                    dry_run,
                    network_config,
                    reprocess,
                    &mut chain,
                )
            })
            .collect::<Result<Vec<SignedEvents>, SentinelError>>()?,
    );
    info!("finished processing {network_id} submission material");

    let r = ProcessorOutput::new(network_id, batch.get_last_block_num()?, signed_events)?;
    Ok(r)
}
