use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::{
            RedeemParams,
            ProcessedTxIds,
        },
    },
};

fn filter_out_already_processed_txs(
    redeem_params: &Vec<RedeemParams>,
    processed_tx_ids: &ProcessedTxIds,
) -> Result<Vec<RedeemParams>> {
    Ok(
        redeem_params
            .iter()
            .filter(|params|
                !processed_tx_ids
                    .contains(&params.originating_tx_id.to_string())
            )
            .cloned()
            .collect::<Vec<RedeemParams>>()
    )
}

pub fn filter_out_already_processed_tx_ids_from_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out already processed tx IDs...");
    filter_out_already_processed_txs(
        &state.redeem_params,
        &state.processed_tx_ids,
    )
        .and_then(|filtered_params| state.add_redeem_params(filtered_params))
}
