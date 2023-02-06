use crate::{chains::btc::btc_block::BtcBlockInDbFormat, state::BtcState, traits::DatabaseInterface, types::Result};

pub fn create_btc_block_in_db_format_and_put_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Creating DB formatted BTC block from block in state...");
    let block = state.get_btc_block_and_id()?.clone();
    let tx_infos = if state.tx_infos.is_empty() {
        None
    } else {
        Some(state.tx_infos.clone())
    };
    let extra_data = vec![];
    state.add_btc_block_in_db_format(BtcBlockInDbFormat::new(
        block.height,
        block.id,
        extra_data,
        None,
        None,
        tx_infos,
        block.block.header.prev_blockhash,
    ))
}
