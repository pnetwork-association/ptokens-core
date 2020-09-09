use crate::{
    btc_on_eth::btc::btc_state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn set_any_sender_flag_in_state<D>(
    state: BtcState<D>
) -> Result<BtcState<D>>
    where D: DatabaseInterface,
{
    info!("✔ Setting AnySender flag in BTC state...");
    let any_sender = state.get_btc_submission_json()?.any_sender;
    state.add_any_sender_flag(any_sender)
}
