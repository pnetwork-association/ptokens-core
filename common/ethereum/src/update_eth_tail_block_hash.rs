use common::{traits::DatabaseInterface, types::Result};

use crate::{
    eth_constants::ETH_TAIL_LENGTH,
    eth_database_utils::EthDbUtilsExt,
    eth_submission_material::EthSubmissionMaterial,
    EthState,
};

fn does_tail_block_require_updating<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    calculated_tail_block: &EthSubmissionMaterial,
) -> Result<bool> {
    info!(
        "✔ Checking if {} tail block needs updating...",
        if db_utils.get_is_for_eth() { "ETH" } else { "EVM" }
    );
    db_utils
        .get_eth_tail_block_from_db()
        .and_then(|db_tail_block| Ok(db_tail_block.get_block_number()? < calculated_tail_block.get_block_number()?))
}

pub fn maybe_update_eth_tail_block_hash<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<()> {
    let side = if db_utils.get_is_for_eth() { "ETH" } else { "EVM" };
    info!("maybe updating {side} tail block hash...");
    let canon_to_tip_length = db_utils.get_eth_canon_to_tip_length_from_db()?;
    let n = canon_to_tip_length + ETH_TAIL_LENGTH;
    db_utils
        .get_eth_latest_block_from_db()
        .and_then(|latest_eth_block| {
            info!("searching for {side} tail block {n} blocks back from tip...");
            db_utils.maybe_get_nth_ancestor_eth_submission_material(&latest_eth_block.get_block_hash()?, n)
        })
        .and_then(|maybe_ancester_block_and_id| match maybe_ancester_block_and_id {
            None => {
                info!("no {n}th ancestor block in db ∴ not updating tail block hash!");
                Ok(())
            },
            Some(ancestor_block) => {
                info!("{side} {n}th ancestor block found");
                if matches!(does_tail_block_require_updating(db_utils, &ancestor_block), Ok(true)) {
                    info!("updating {side} tail block...");
                    db_utils.put_eth_tail_block_hash_in_db(&ancestor_block.get_block_hash()?)
                } else {
                    info!("{side} tail block does not require updating");
                    Ok(())
                }
            },
        })
}

pub fn maybe_update_eth_tail_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_update_eth_tail_block_hash(&state.eth_db_utils).and(Ok(state))
}

pub fn maybe_update_evm_tail_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_update_eth_tail_block_hash(&state.evm_db_utils).and(Ok(state))
}
