use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::btc::btc_state::BtcState,
};

pub fn maybe_filter_out_value_too_low_btc_on_eos_minting_params_in_state<D: DatabaseInterface>(
    state: BtcState<D>
) -> Result<BtcState<D>> {
    state
        .btc_on_eos_minting_params
        .filter_out_value_too_low()
        .and_then(|params| state.replace_btc_on_eos_minting_params(params))
}
