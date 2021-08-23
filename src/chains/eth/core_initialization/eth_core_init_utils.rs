use crate::{
    chains::eth::{eth_chain_id::EthChainId, eth_state::EthState},
    traits::DatabaseInterface,
    types::Result,
};

pub fn put_eth_tail_block_hash_in_db_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Putting ETH tail block has in db...");
    state
        .eth_db_utils
        .put_eth_tail_block_hash_in_db(&state.get_eth_submission_material()?.get_block_hash()?)
        .and(Ok(state))
}

fn set_hash_from_block_in_state<'a, D: DatabaseInterface>(
    state: EthState<'a, D>,
    hash_type: &str,
) -> Result<EthState<'a, D>> {
    let hash = &state.get_eth_submission_material()?.get_block_hash()?;
    match hash_type {
        "canon" => {
            info!("✔ Initializating ETH canon block hash...");
            state.eth_db_utils.put_eth_canon_block_hash_in_db(hash)
        },
        "latest" => {
            info!("✔ Initializating ETH latest block hash...");
            state.eth_db_utils.put_eth_latest_block_hash_in_db(hash)
        },
        "anchor" => {
            info!("✔ Initializating ETH anchor block hash...");
            state.eth_db_utils.put_eth_anchor_block_hash_in_db(hash)
        },
        _ => Err("✘ Hash type not recognized!".into()),
    }?;
    Ok(state)
}

pub fn set_eth_latest_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(state, "latest")
}

pub fn set_eth_anchor_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(state, "anchor")
}

pub fn set_eth_canon_block_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    set_hash_from_block_in_state(state, "canon")
}

pub fn put_canon_to_tip_length_in_db_and_return_state<D: DatabaseInterface>(
    canon_to_tip_length: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    state
        .eth_db_utils
        .put_eth_canon_to_tip_length_in_db(canon_to_tip_length)
        .and(Ok(state))
}

pub fn put_eth_chain_id_in_db_and_return_state<'a, D: DatabaseInterface>(
    chain_id: &EthChainId,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    info!("✔ Putting ETH chain ID of {} in db...", chain_id,);
    state.eth_db_utils.put_eth_chain_id_in_db(chain_id).and(Ok(state))
}

pub fn put_eth_gas_price_in_db_and_return_state<D: DatabaseInterface>(
    gas_price: u64,
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Putting ETH gas price of {} in db...", gas_price);
    state.eth_db_utils.put_eth_gas_price_in_db(gas_price).and(Ok(state))
}

pub fn put_eth_account_nonce_in_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
    nonce: u64,
) -> Result<EthState<D>> {
    info!("✔ Putting ETH account nonce of 1 in db...");
    state.eth_db_utils.put_eth_account_nonce_in_db(nonce).and(Ok(state))
}

pub fn remove_receipts_from_block_in_state<D: DatabaseInterface>(
    // ∵ there shouldn't be relevant txs!
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Removing receipts from ETH block in state...");
    let submission_material_with_no_receipts = state.get_eth_submission_material()?.remove_receipts();
    state.update_eth_submission_material(submission_material_with_no_receipts)
}

pub fn add_eth_block_to_db_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Adding ETH block and receipts to db...",);
    state
        .eth_db_utils
        .put_eth_submission_material_in_db(state.get_eth_submission_material()?)
        .and(Ok(state))
}

pub fn put_any_sender_nonce_in_db_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Putting AnySender nonce of 0 in db...");
    state.eth_db_utils.put_any_sender_nonce_in_db(0).and(Ok(state))
}
