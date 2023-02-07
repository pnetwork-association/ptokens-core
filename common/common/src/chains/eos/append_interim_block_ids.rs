use eos_chain::Checksum256;

use crate::{chains::eos::eos_merkle_utils::Incremerkle, state::EosState, traits::DatabaseInterface, types::Result};

pub fn append_block_ids_to_incremerkle(mut incremerkle: Incremerkle, block_ids: &[Checksum256]) -> Result<Incremerkle> {
    block_ids.iter().for_each(|id| {
        incremerkle.append(*id).ok();
    });
    Ok(incremerkle)
}

pub fn append_interim_block_ids_to_incremerkle_in_state<D>(state: EosState<D>) -> Result<EosState<D>>
where
    D: DatabaseInterface,
{
    info!("✔ Appending interim block IDs to incremerkle...");
    append_block_ids_to_incremerkle(state.incremerkle.clone(), &state.interim_block_ids)
        .map(|incremerkle| state.add_incremerkle(incremerkle))
}
