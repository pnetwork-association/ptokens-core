use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::btc::{
        btc_state::BtcState,
        btc_types::BtcBlockInDbFormat,
    },
};

pub fn create_btc_block_in_db_format_and_put_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Creating DB formatted BTC block from block in state...");
    let block_in_state = &state.get_btc_block_and_id()?;
    BtcBlockInDbFormat::new(
        block_in_state.height,
        block_in_state.id,
        block_in_state.block.clone(),
        vec![], // NOTE: As yet unused `extra_data` param
        Some(state.btc_on_eos_minting_params.clone()),
        None,
    )
        .and_then(|block_in_db_format| state.add_btc_block_in_db_format(block_in_db_format))
}
