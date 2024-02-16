use common::{traits::DatabaseInterface, types::Result};
use common_chain_ids::EosChainId;
use common_eos::{
    filter_for_proofs_with_action_name,
    EosState,
    REDEEM_ACTION_NAME as V1_REDEEM_ACTION_NAME,
    V2_REDEEM_ACTION_NAME,
};

pub fn maybe_filter_for_relevant_redeem_actions<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    // NOTE: We care about ALL `redeem2` actions...
    info!("✔ Filtering for `redeem2` actions...");
    let v2_redeem_proofs = filter_for_proofs_with_action_name(&state.action_proofs, V2_REDEEM_ACTION_NAME)?;
    debug!("found {} v2 redeem proofs", v2_redeem_proofs.len());

    let eos_cid = state.eos_db_utils.get_eos_chain_id_from_db()?;

    let v1_proofs = match eos_cid {
        EosChainId::UltraMainnet | EosChainId::EosMainnet => {
            // NOTE: However we also occasionally want some v1 `redeem` actions, for custom handling,
            // in order to facilitate v1->v2 bridge migrations in cases where it's not so easy on chain.
            let proofs = filter_for_proofs_with_action_name(&state.action_proofs, V1_REDEEM_ACTION_NAME)?;
            let num_proofs = proofs.len();

            debug!("found {num_proofs} v1 redeem proofs for eos chain id: {eos_cid} ");
            proofs
        },
        other => {
            warn!("no custom handling for redeem proofs from eos chain id: {other}");
            vec![]
        },
    };

    state.replace_action_proofs([v1_proofs, v2_redeem_proofs].concat())
}
