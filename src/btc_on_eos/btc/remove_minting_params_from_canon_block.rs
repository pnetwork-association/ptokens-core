use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::btc::{
        btc_state::BtcState,
        btc_database_utils::{
            put_btc_canon_block_in_db,
            get_btc_canon_block_from_db,
        },
    },
};

fn remove_minting_params_from_canon_block<D: DatabaseInterface>(db: &D) -> Result<()> {
    get_btc_canon_block_from_db(db)
        .and_then(|canon_block| canon_block.remove_minting_params())
        .and_then(|canon_block| put_btc_canon_block_in_db(db, &canon_block))
}

pub fn remove_minting_params_from_canon_block_and_return_state<D>(
    state: BtcState<D>
) -> Result<BtcState<D>>
    where D: DatabaseInterface
{
    info!("✔ Removing minting params from canon block...");
    remove_minting_params_from_canon_block(&state.db).and(Ok(state))
}
