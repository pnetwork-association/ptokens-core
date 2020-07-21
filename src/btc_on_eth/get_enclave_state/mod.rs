use crate::{
    types::Result,
    traits::DatabaseInterface,
    constants::{
        DEBUG_MODE,
        DB_KEY_PREFIX,
        CORE_IS_VALIDATING,
    },
    chains::{
        eth::eth_constants::ETH_TAIL_LENGTH,
        btc::{
            btc_constants::BTC_TAIL_LENGTH,
            utxo_manager::utxo_database_utils::{
                get_utxo_nonce_from_db,
                get_total_utxo_balance_from_db,
                get_total_number_of_utxos_from_db,
            },
        },
    },
    btc_on_eth::{
        constants::{
            SAFE_ETH_ADDRESS,
            SAFE_BTC_ADDRESS,
        },
        eth::{
            get_linker_hash::{
                get_linker_hash_or_genesis_hash as get_eth_linker_hash
            },
            eth_database_utils::{
                get_eth_gas_price_from_db,
                get_eth_tail_block_from_db,
                get_eth_canon_block_from_db,
                get_eth_latest_block_from_db,
                get_any_sender_nonce_from_db,
                get_eth_anchor_block_from_db,
                get_eth_account_nonce_from_db,
                get_public_eth_address_from_db,
                get_eth_canon_to_tip_length_from_db,
                get_erc777_contract_address_from_db,
                get_erc777_proxy_contract_address_from_db,
            },
        },
        btc::{
            update_btc_linker_hash::{
                get_linker_hash_or_genesis_hash as get_btc_linker_hash,
            },
            btc_database_utils::{
                get_btc_fee_from_db,
                get_btc_network_from_db,
                get_btc_address_from_db,
                get_btc_tail_block_from_db,
                get_btc_difficulty_from_db,
                get_btc_private_key_from_db,
                get_btc_canon_block_from_db,
                get_btc_latest_block_from_db,
                get_btc_anchor_block_from_db,
                get_btc_canon_to_tip_length_from_db,
            },
        },
        check_core_is_initialized::check_core_is_initialized,
    },
};

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    debug_mode: bool,
    eth_gas_price: u64,
    btc_difficulty: u64,
    btc_network: String,
    eth_address: String,
    btc_address: String,
    btc_utxo_nonce: u64,
    btc_tail_length: u64,
    eth_tail_length: u64,
    any_sender_nonce: u64,
    db_key_prefix: String,
    btc_public_key: String,
    btc_sats_per_byte: u64,
    eth_account_nonce: u64,
    eth_linker_hash: String,
    btc_linker_hash: String,
    core_is_validating: bool,
    btc_number_of_utxos: u64,
    btc_safe_address: String,
    eth_safe_address: String,
    btc_utxo_total_value: u64,
    eth_tail_block_hash: String,
    btc_tail_block_hash: String,
    btc_canon_block_hash: String,
    btc_tail_block_number: u64,
    eth_tail_block_number: usize,
    eth_canon_block_hash: String,
    eth_anchor_block_hash: String,
    eth_latest_block_hash: String,
    btc_latest_block_hash: String,
    btc_canon_block_number: u64,
    eth_canon_block_number: usize,
    btc_anchor_block_hash: String,
    eth_anchor_block_number: usize,
    smart_contract_address: String,
    btc_latest_block_number: u64,
    eth_canon_to_tip_length: u64,
    btc_anchor_block_number: u64,
    btc_canon_to_tip_length: u64,
    eth_latest_block_number: usize,
    erc777_proxy_contract_address: String,
}

pub fn get_enclave_state<D>(
    db: D
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Getting enclave state...");
    check_core_is_initialized(&db)
        .and_then(|_| {
            let eth_tail_block = get_eth_tail_block_from_db(&db)?;
            let btc_tail_block = get_btc_tail_block_from_db(&db)?;
            let eth_canon_block = get_eth_canon_block_from_db(&db)?;
            let btc_canon_block = get_btc_canon_block_from_db(&db)?;
            let btc_private_key = get_btc_private_key_from_db(&db)?;
            let eth_anchor_block = get_eth_anchor_block_from_db(&db)?;
            let btc_anchor_block = get_btc_anchor_block_from_db(&db)?;
            let eth_latest_block = get_eth_latest_block_from_db(&db)?;
            let btc_latest_block = get_btc_latest_block_from_db(&db)?;
            let btc_public_key_hex = hex::encode(&btc_private_key.to_public_key_slice().to_vec());
            Ok(serde_json::to_string(
                &EnclaveState {
                    debug_mode: DEBUG_MODE,
                    btc_tail_length: BTC_TAIL_LENGTH,
                    eth_tail_length: ETH_TAIL_LENGTH,
                    btc_public_key: btc_public_key_hex,
                    core_is_validating: CORE_IS_VALIDATING,
                    db_key_prefix: DB_KEY_PREFIX.to_string(),
                    btc_address: get_btc_address_from_db(&db)?,
                    btc_utxo_nonce: get_utxo_nonce_from_db(&db)?,
                    btc_tail_block_number: btc_tail_block.height,
                    btc_sats_per_byte: get_btc_fee_from_db(&db)?,
                    eth_gas_price: get_eth_gas_price_from_db(&db)?,
                    btc_canon_block_number: btc_canon_block.height,
                    btc_safe_address: SAFE_BTC_ADDRESS.to_string(),
                    btc_latest_block_number: btc_latest_block.height,
                    btc_difficulty: get_btc_difficulty_from_db(&db)?,
                    btc_anchor_block_number: btc_anchor_block.height,
                    btc_tail_block_hash: btc_tail_block.id.to_string(),
                    btc_canon_block_hash: btc_canon_block.id.to_string(),
                    any_sender_nonce: get_any_sender_nonce_from_db(&db)?,
                    btc_latest_block_hash: btc_latest_block.id.to_string(),
                    btc_anchor_block_hash: btc_anchor_block.id.to_string(),
                    btc_linker_hash: get_btc_linker_hash(&db)?.to_string(),
                    btc_network: get_btc_network_from_db(&db)?.to_string(),
                    eth_account_nonce: get_eth_account_nonce_from_db(&db)?,
                    eth_safe_address: hex::encode(SAFE_ETH_ADDRESS.as_bytes()),
                    btc_utxo_total_value: get_total_utxo_balance_from_db(&db)?,
                    btc_number_of_utxos: get_total_number_of_utxos_from_db(&db)?,
                    eth_tail_block_number: eth_tail_block.block.number.as_usize(),
                    eth_canon_block_number: eth_canon_block.block.number.as_usize(),
                    eth_anchor_block_number: eth_anchor_block.block.number.as_usize(),
                    eth_latest_block_number: eth_latest_block.block.number.as_usize(),
                    eth_linker_hash: hex::encode(get_eth_linker_hash(&db)?.as_bytes()),
                    eth_canon_to_tip_length: get_eth_canon_to_tip_length_from_db(&db)?,
                    btc_canon_to_tip_length: get_btc_canon_to_tip_length_from_db(&db)?,
                    eth_tail_block_hash: hex::encode(eth_tail_block.block.hash.as_bytes()),
                    eth_canon_block_hash: hex::encode(eth_canon_block.block.hash.as_bytes()),
                    eth_anchor_block_hash: hex::encode(eth_anchor_block.block.hash.as_bytes()),
                    eth_latest_block_hash: hex::encode(eth_latest_block.block.hash.as_bytes()),
                    eth_address: hex::encode(get_public_eth_address_from_db(&db)?.as_bytes()),
                    erc777_proxy_contract_address: hex::encode(get_erc777_proxy_contract_address_from_db(&db)?),
                    smart_contract_address: hex::encode(get_erc777_contract_address_from_db(&db)?.as_bytes()),
                }
            )?)
        })
}
